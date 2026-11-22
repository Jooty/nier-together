use core::ffi::c_void;
use core::ptr;
use std::collections::HashMap;
use std::panic;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock, PoisonError};

use replicant_game::GameMemory;
use replicant_game::enemy_bullets::{
    SpriteAddView, SpriteRefusalTally, TracedDraw, open_view_frustum_planes, sprite_refusal,
};
use replicant_game::offsets::{
    BULLET_MANAGER_DRAW_FUNCTION_RVA, BULLET_SPRITE_BATCH_ADD_FUNCTION_RVA,
    BULLET_SPRITE_BATCH_ENTRIES_THIS_FRAME_OFFSET, BULLET_SPRITE_BATCH_FREE_ENTRY_POINTER_OFFSET,
    BULLET_SPRITE_BATCH_RVA, DRAW_CAMERA_MATRIX_OFFSET, DRAW_CAMERA_RVA,
    SPRITE_NEAREST_CAMERA_DISTANCE_RVA, SPRITE_VIEW_FRUSTUM_RVA,
    SPRITE_VIEW_FRUSTUM_TEST_FUNCTION_RVA, VIEW_FRUSTUM_PLANES_BLOCK_LENGTH,
    VIEW_FRUSTUM_PLANES_BLOCK_OFFSET,
};
use replicant_proto::BulletSprite;

use crate::build_gate::VerifiedBuild;
use crate::game_hooks::hook_address;
use crate::memory::InProcessMemory;

const SPRITE_POSITION_W: f32 = 1.0;
const CAMERA_POSITION_OFFSETS_IN_MATRIX: [u64; 3] = [0xC, 0x1C, 0x2C];
const MOST_HOST_SPRITES_ADDED_WITH_THE_FRUSTUM_OPEN: usize = 600;

static ORIGINAL_DRAW: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static SPRITES_TO_DRAW: Mutex<Vec<SpriteToDraw>> = Mutex::new(Vec::new());
static DRAWS: AtomicU32 = AtomicU32::new(0);
static DRAWS_OF_ANY_KIND: AtomicU32 = AtomicU32::new(0);
static DRAWS_WITHOUT_A_BATCH: AtomicU32 = AtomicU32::new(0);
static SPRITES_HANDED_TO_THE_BATCH: AtomicU32 = AtomicU32::new(0);
static SPRITES_THE_BATCH_TOOK: AtomicU32 = AtomicU32::new(0);
static SPRITES_OFF_SCREEN_BY_THE_GAMES_OWN_TEST: AtomicU32 = AtomicU32::new(0);
static OFF_SCREEN_SAMPLE_LINE: Mutex<Option<String>> = Mutex::new(None);
static REFUSALS: LazyLock<Mutex<SpriteRefusalTally>> = LazyLock::new(Mutex::default);
static TRACED_DRAWS: LazyLock<Mutex<HashMap<u16, TracedDraw>>> = LazyLock::new(Mutex::default);

type DrawBullets = unsafe extern "system" fn(*mut c_void);
type AddSprite =
    unsafe extern "system" fn(*mut c_void, i32, i32, *const f32, f32, f32, f32, *const c_void);
type ViewFrustumTest = unsafe extern "system" fn(*const c_void, *const f32, f32) -> u8;

#[repr(align(16))]
struct AlignedPosition([f32; 4]);

#[derive(Debug, Clone, Copy)]
pub(super) struct SpriteToDraw {
    pub slot: u16,
    pub traced: bool,
    pub position: [f32; 3],
    pub sprite: BulletSprite,
    pub frame: u16,
}

/// What the bullet draw hook did since the last call, for the guest's log.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct SpriteDrawCounts {
    pub draws_of_any_kind: u32,
    pub draws: u32,
    pub draws_without_a_batch: u32,
    pub handed: u32,
    pub taken: u32,
    pub off_screen_by_the_games_own_test: u32,
}

struct OpenedFrustum {
    memory: InProcessMemory,
    address: u64,
    closed: [u8; VIEW_FRUSTUM_PLANES_BLOCK_LENGTH],
}

struct DrawView {
    frustum: usize,
    frustum_test: usize,
    camera_position: Option<[f32; 3]>,
    nearest_distance_squared: f32,
}

impl OpenedFrustum {
    fn open(build: &VerifiedBuild) -> Option<Self> {
        let mut memory = InProcessMemory::new(build);
        let address = build.module_base()
            + u64::from(SPRITE_VIEW_FRUSTUM_RVA)
            + VIEW_FRUSTUM_PLANES_BLOCK_OFFSET;
        let closed = memory
            .read_array::<VIEW_FRUSTUM_PLANES_BLOCK_LENGTH>(address)
            .ok()?;
        let mut opened = closed;
        open_view_frustum_planes(&mut opened);
        memory.write(address, &opened).ok()?;
        Some(Self {
            memory,
            address,
            closed,
        })
    }
}

impl Drop for OpenedFrustum {
    fn drop(&mut self) {
        let _ = self.memory.write(self.address, &self.closed);
    }
}

impl DrawView {
    fn read(memory: &InProcessMemory, build: &VerifiedBuild) -> Self {
        let base = build.module_base();
        let matrix = base + u64::from(DRAW_CAMERA_RVA) + DRAW_CAMERA_MATRIX_OFFSET;
        let camera_position = (|| {
            Some([
                memory
                    .read_f32(matrix + CAMERA_POSITION_OFFSETS_IN_MATRIX[0])
                    .ok()?,
                memory
                    .read_f32(matrix + CAMERA_POSITION_OFFSETS_IN_MATRIX[1])
                    .ok()?,
                memory
                    .read_f32(matrix + CAMERA_POSITION_OFFSETS_IN_MATRIX[2])
                    .ok()?,
            ])
        })();
        let nearest = memory
            .read_f32(base + u64::from(SPRITE_NEAREST_CAMERA_DISTANCE_RVA))
            .unwrap_or(0.0);
        Self {
            frustum: (base + u64::from(SPRITE_VIEW_FRUSTUM_RVA)) as usize,
            frustum_test: (base + u64::from(SPRITE_VIEW_FRUSTUM_TEST_FUNCTION_RVA)) as usize,
            camera_position,
            nearest_distance_squared: nearest * nearest,
        }
    }

    fn distance_squared_to(&self, position: [f32; 3]) -> f32 {
        let Some(camera) = self.camera_position else {
            return f32::INFINITY;
        };
        (0..3)
            .map(|axis| (camera[axis] - position[axis]).powi(2))
            .sum()
    }

    fn sprite_in_view(&self, position: &AlignedPosition, size: f32) -> bool {
        #[expect(unsafe_code, reason = "calls the game's view frustum test")]
        // SAFETY: the RVA starts the function (frustum, position of at least three floats,
        // radius -> bool) in the verified build, as the sprite batch's add function calls it
        // with this frustum; the position outlives the call and the function only reads.
        unsafe {
            core::mem::transmute::<usize, ViewFrustumTest>(self.frustum_test)(
                self.frustum as *const c_void,
                position.0.as_ptr(),
                size,
            ) != 0
        }
    }
}

pub fn install(build: VerifiedBuild) {
    if HOOKED_BUILD.set(build).is_err() {
        return;
    }
    let address = (build.module_base() + u64::from(BULLET_MANAGER_DRAW_FUNCTION_RVA)) as usize;
    #[expect(unsafe_code, reason = "hooks the bullet manager's draw by its address")]
    // SAFETY: the RVA starts the bullet manager's draw (manager, void) in the verified build,
    // and the replacement has the same signature.
    unsafe {
        hook_address(
            "BulletMan::Draw",
            address,
            draw_detour as *const () as *mut c_void,
            &ORIGINAL_DRAW,
        );
    }
}

pub(super) fn set_sprites_to_draw(sprites: Vec<SpriteToDraw>) {
    *SPRITES_TO_DRAW
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = sprites;
}

pub(super) fn take_sprite_refusal_line() -> Option<String> {
    REFUSALS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take_line()
}

pub(super) fn last_traced_draw(slot: u16) -> Option<TracedDraw> {
    TRACED_DRAWS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&slot)
        .copied()
}

pub(super) fn take_sprite_draw_counts() -> SpriteDrawCounts {
    SpriteDrawCounts {
        draws_of_any_kind: DRAWS_OF_ANY_KIND.swap(0, Ordering::Relaxed),
        draws: DRAWS.swap(0, Ordering::Relaxed),
        draws_without_a_batch: DRAWS_WITHOUT_A_BATCH.swap(0, Ordering::Relaxed),
        handed: SPRITES_HANDED_TO_THE_BATCH.swap(0, Ordering::Relaxed),
        taken: SPRITES_THE_BATCH_TOOK.swap(0, Ordering::Relaxed),
        off_screen_by_the_games_own_test: SPRITES_OFF_SCREEN_BY_THE_GAMES_OWN_TEST
            .swap(0, Ordering::Relaxed),
    }
}

pub(super) fn take_off_screen_sample_line() -> Option<String> {
    OFF_SCREEN_SAMPLE_LINE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take()
}

#[expect(
    unsafe_code,
    reason = "calls the original bullet draw through the trampoline MinHook built"
)]
extern "system" fn draw_detour(manager: *mut c_void) {
    // The game's own bullets add their sprites inside the original; the batch is flushed and
    // emptied at its end, so the host's sprites go in first.
    DRAWS_OF_ANY_KIND.fetch_add(1, Ordering::Relaxed);
    let _ = panic::catch_unwind(add_host_sprites);
    let original = ORIGINAL_DRAW.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    // SAFETY: the trampoline continues into the original draw, which has this exact signature.
    unsafe {
        core::mem::transmute::<*mut c_void, DrawBullets>(original)(manager);
    }
}

fn add_host_sprites() {
    let Some(build) = HOOKED_BUILD.get() else {
        return;
    };
    let sprites = SPRITES_TO_DRAW
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    if sprites.is_empty() {
        return;
    }
    DRAWS.fetch_add(1, Ordering::Relaxed);
    let memory = InProcessMemory::new(build);
    let batch = build.module_base() + u64::from(BULLET_SPRITE_BATCH_RVA);
    let entries = |memory: &InProcessMemory| {
        memory
            .read_i32(batch + BULLET_SPRITE_BATCH_ENTRIES_THIS_FRAME_OFFSET)
            .unwrap_or(0)
    };
    // The batch has entries to hand out only between a phase's set-up and its tear-down.
    let set_up = memory
        .read_u64(batch + BULLET_SPRITE_BATCH_FREE_ENTRY_POINTER_OFFSET)
        .is_ok_and(|free_entry| free_entry != 0);
    if !set_up {
        DRAWS_WITHOUT_A_BATCH.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let function = (build.module_base() + u64::from(BULLET_SPRITE_BATCH_ADD_FUNCTION_RVA)) as usize;
    let view = DrawView::read(&memory, build);
    let on_screen_by_the_games_own_test: Vec<bool> = sprites
        .iter()
        .map(|drawn| view.sprite_in_view(&aligned_position_of(drawn), drawn.sprite.size))
        .collect();
    note_sprites_off_screen(&sprites, &on_screen_by_the_games_own_test, &view);
    let opened_frustum = (sprites.len() <= MOST_HOST_SPRITES_ADDED_WITH_THE_FRUSTUM_OPEN)
        .then(|| OpenedFrustum::open(build))
        .flatten();
    let frustum_is_open = opened_frustum.is_some();
    let mut taken_total = 0_u32;
    let mut traced_draws = HashMap::new();
    for (index, drawn) in sprites.iter().enumerate() {
        let position = aligned_position_of(drawn);
        let before = entries(&memory);
        let batch_full = free_entry_is_null(&memory, batch);
        #[expect(unsafe_code, reason = "calls the game's add-to-sprite-batch function")]
        // SAFETY: the RVA starts the function (batch, sprite kind, frame, position of four
        // floats, alpha, size, angle, light or null -> void) in the verified build, as the sprite
        // object's own draw calls it; the position is 16-byte aligned and outlives the call.
        unsafe {
            core::mem::transmute::<usize, AddSprite>(function)(
                batch as usize as *mut c_void,
                i32::from(drawn.sprite.kind),
                i32::from(drawn.frame),
                position.0.as_ptr(),
                drawn.sprite.alpha,
                drawn.sprite.size,
                drawn.sprite.angle,
                ptr::null(),
            );
        }
        let draw = if entries(&memory) > before {
            taken_total += 1;
            TracedDraw::Taken
        } else {
            let refusal = sprite_refusal(SpriteAddView {
                batch_full,
                in_view: batch_full || frustum_is_open || on_screen_by_the_games_own_test[index],
                camera_distance_squared: view.distance_squared_to(drawn.position),
                nearest_distance_squared: view.nearest_distance_squared,
            });
            REFUSALS
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .note(refusal);
            TracedDraw::Refused(refusal)
        };
        if drawn.traced {
            traced_draws.insert(drawn.slot, draw);
        }
    }
    *TRACED_DRAWS.lock().unwrap_or_else(PoisonError::into_inner) = traced_draws;
    SPRITES_HANDED_TO_THE_BATCH.fetch_add(sprites.len() as u32, Ordering::Relaxed);
    SPRITES_THE_BATCH_TOOK.fetch_add(taken_total, Ordering::Relaxed);
}

fn aligned_position_of(drawn: &SpriteToDraw) -> AlignedPosition {
    AlignedPosition([
        drawn.position[0],
        drawn.position[1],
        drawn.position[2],
        SPRITE_POSITION_W,
    ])
}

fn note_sprites_off_screen(sprites: &[SpriteToDraw], on_screen: &[bool], view: &DrawView) {
    let Some(first_off_screen) = on_screen.iter().position(|visible| !visible) else {
        return;
    };
    let off_screen = on_screen.iter().filter(|visible| !**visible).count();
    SPRITES_OFF_SCREEN_BY_THE_GAMES_OWN_TEST.fetch_add(off_screen as u32, Ordering::Relaxed);
    let drawn = &sprites[first_off_screen];
    let mut sample = OFF_SCREEN_SAMPLE_LINE
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if sample.is_none() {
        *sample = Some(format!(
            "enemy bullets: a sprite the game's own view test calls off screen: size {:.2} at ({:.1}, {:.1}, {:.1}), the draw camera at {:?}, {} of {} handed this draw",
            drawn.sprite.size,
            drawn.position[0],
            drawn.position[1],
            drawn.position[2],
            view.camera_position,
            off_screen,
            sprites.len()
        ));
    }
}

fn free_entry_is_null(memory: &InProcessMemory, batch: u64) -> bool {
    memory
        .read_u64(batch + BULLET_SPRITE_BATCH_FREE_ENTRY_POINTER_OFFSET)
        .is_ok_and(|free_entry| free_entry == 0)
}
