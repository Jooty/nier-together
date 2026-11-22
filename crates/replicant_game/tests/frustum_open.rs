use replicant_game::enemy_bullets::open_view_frustum_planes;
use replicant_game::offsets::{
    VIEW_FRUSTUM_NEAR_PLANE_NORMAL_OFFSET, VIEW_FRUSTUM_NEAR_PLANE_OFFSET_OFFSET,
    VIEW_FRUSTUM_PLANES_BLOCK_LENGTH, VIEW_FRUSTUM_PLANES_BLOCK_OFFSET,
    VIEW_FRUSTUM_SIDE_PLANE_COUNT, VIEW_FRUSTUM_SIDE_PLANE_STRIDE, VIEW_FRUSTUM_SIDE_PLANES_OFFSET,
};

const NEAR_DISTANCE: f32 = 0.1;
const UNTOUCHED_BYTE: u8 = 0xAB;

fn float_at(block: &[u8; VIEW_FRUSTUM_PLANES_BLOCK_LENGTH], offset: u64) -> f32 {
    let at = (offset - VIEW_FRUSTUM_PLANES_BLOCK_OFFSET) as usize;
    f32::from_le_bytes(block[at..at + 4].try_into().expect("four bytes"))
}

fn set_float(block: &mut [u8; VIEW_FRUSTUM_PLANES_BLOCK_LENGTH], offset: u64, value: f32) {
    let at = (offset - VIEW_FRUSTUM_PLANES_BLOCK_OFFSET) as usize;
    block[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn sphere_visible(
    block: &[u8; VIEW_FRUSTUM_PLANES_BLOCK_LENGTH],
    position: [f32; 3],
    radius: f32,
) -> bool {
    let near_normal = VIEW_FRUSTUM_NEAR_PLANE_NORMAL_OFFSET;
    let near: f32 = (0..3)
        .map(|axis| float_at(block, near_normal + 4 * axis) * position[axis as usize])
        .sum::<f32>()
        + float_at(block, VIEW_FRUSTUM_NEAR_PLANE_OFFSET_OFFSET)
        - NEAR_DISTANCE;
    if near + radius < 0.0 {
        return false;
    }
    (0..VIEW_FRUSTUM_SIDE_PLANE_COUNT).all(|plane| {
        let start = VIEW_FRUSTUM_SIDE_PLANES_OFFSET + plane * VIEW_FRUSTUM_SIDE_PLANE_STRIDE;
        let distance: f32 = (0..3)
            .map(|axis| float_at(block, start + 4 * axis) * position[axis as usize])
            .sum::<f32>()
            + float_at(block, start + 12);
        distance + radius >= 0.0
    })
}

fn camera_looking_down_the_x_axis() -> [u8; VIEW_FRUSTUM_PLANES_BLOCK_LENGTH] {
    let mut block = [UNTOUCHED_BYTE; VIEW_FRUSTUM_PLANES_BLOCK_LENGTH];
    for (axis, value) in [1.0_f32, 0.0, 0.0].into_iter().enumerate() {
        set_float(
            &mut block,
            VIEW_FRUSTUM_NEAR_PLANE_NORMAL_OFFSET + 4 * axis as u64,
            value,
        );
    }
    set_float(&mut block, VIEW_FRUSTUM_NEAR_PLANE_OFFSET_OFFSET, 0.0);
    for plane in 0..VIEW_FRUSTUM_SIDE_PLANE_COUNT {
        let start = VIEW_FRUSTUM_SIDE_PLANES_OFFSET + plane * VIEW_FRUSTUM_SIDE_PLANE_STRIDE;
        for (axis, value) in [1.0_f32, 0.0, 0.0].into_iter().enumerate() {
            set_float(&mut block, start + 4 * axis as u64, value);
        }
        set_float(&mut block, start + 12, 0.0);
    }
    block
}

#[test]
fn a_sphere_behind_the_camera_is_off_screen_until_the_planes_are_opened() {
    let mut block = camera_looking_down_the_x_axis();
    let behind = [-50.0, 3.0, 4.0];
    assert!(sphere_visible(&block, [50.0, 3.0, 4.0], 1.5));
    assert!(!sphere_visible(&block, behind, 1.5));
    open_view_frustum_planes(&mut block);
    assert!(sphere_visible(&block, behind, 1.5));
    assert!(sphere_visible(&block, [-5000.0, 900.0, -700.0], 0.0));
}

#[test]
fn opening_the_planes_leaves_every_other_byte_of_the_block_alone() {
    let closed = camera_looking_down_the_x_axis();
    let mut opened = closed;
    open_view_frustum_planes(&mut opened);
    let near_normal =
        (VIEW_FRUSTUM_NEAR_PLANE_NORMAL_OFFSET - VIEW_FRUSTUM_PLANES_BLOCK_OFFSET) as usize;
    let side_planes = (VIEW_FRUSTUM_SIDE_PLANES_OFFSET - VIEW_FRUSTUM_PLANES_BLOCK_OFFSET) as usize;
    let near_offset =
        (VIEW_FRUSTUM_NEAR_PLANE_OFFSET_OFFSET - VIEW_FRUSTUM_PLANES_BLOCK_OFFSET) as usize;
    let side_planes_end =
        side_planes + (VIEW_FRUSTUM_SIDE_PLANE_COUNT * VIEW_FRUSTUM_SIDE_PLANE_STRIDE) as usize;
    for index in 0..VIEW_FRUSTUM_PLANES_BLOCK_LENGTH {
        let inside_a_plane = (near_normal..near_normal + 12).contains(&index)
            || (near_offset..near_offset + 4).contains(&index)
            || (side_planes..side_planes_end).contains(&index);
        if !inside_a_plane {
            assert_eq!(opened[index], closed[index], "byte {index}");
        }
    }
}
