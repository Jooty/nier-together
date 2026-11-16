use replicant_proto::DefeatPick;

use crate::host_led_defeat::{
    GAME_OVER_CONTEXT_RESULT_OFFSET, GAME_OVER_CONTEXT_VFTABLE_RVA,
    GAME_OVER_ROOT_SEQUENCE_VFTABLE_RVA, GAME_OVER_SCENE_ID, GAME_OVER_SEQUENCE_CURSOR_OFFSET,
    GAME_OVER_SEQUENCE_DONE_STATE, GAME_OVER_SEQUENCE_STATE_OFFSET, UI_SCENE_CONTEXT_LIST_OFFSET,
    UI_SCENE_CONTEXT_WALK_LIMIT, UI_SCENE_FIRST_CONTEXT_LINK_OFFSET, cursor_of_pick,
    game_over_result_of_pick,
};
use crate::memory::{GameMemory, MemoryError};
use crate::ui_scenes::{
    UI_LINK_NEXT_OFFSET, UI_MANAGER_FIRST_SCENE_LINK_OFFSET, UI_MANAGER_RVA,
    UI_MANAGER_SCENE_LIST_OFFSET, UI_SCENE_ID_OFFSET, UI_SCENE_WALK_LIMIT,
};

/// One frame of the game's defeat screen sequence (`GameOverRootSequence`): its step (0 takes
/// the buttons, 1 waits on the second item's yes or no dialog, 2 and 3 play the press, 4 writes
/// the result, 5 is done) and the item its cursor stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameOverScreen {
    pub state: i32,
    pub cursor: i32,
}

/// The defeat screen sequence at `sequence`, or `None` when the object there is not one.
pub fn read_game_over_screen<M: GameMemory>(
    memory: &M,
    sequence: u64,
) -> Result<Option<GameOverScreen>, MemoryError> {
    let vftable = memory.module_base() + u64::from(GAME_OVER_ROOT_SEQUENCE_VFTABLE_RVA);
    if memory.read_u64(sequence)? != vftable {
        return Ok(None);
    }
    Ok(Some(GameOverScreen {
        state: memory.read_i32(sequence + GAME_OVER_SEQUENCE_STATE_OFFSET)?,
        cursor: memory.read_i32(sequence + GAME_OVER_SEQUENCE_CURSOR_OFFSET)?,
    }))
}

/// The `GameOverContext` of the open defeat scene (UI scene 14), found as the game's own lookup
/// 0x452CB0 finds it: the scene's context ring, the entry whose table is the context's.
pub fn find_game_over_context<M: GameMemory>(memory: &M) -> Result<Option<u64>, MemoryError> {
    let Some(scene) = find_game_over_scene(memory)? else {
        return Ok(None);
    };
    let ring_end = scene + UI_SCENE_CONTEXT_LIST_OFFSET;
    let vftable = memory.module_base() + u64::from(GAME_OVER_CONTEXT_VFTABLE_RVA);
    let mut link = memory.read_u64(scene + UI_SCENE_FIRST_CONTEXT_LINK_OFFSET)?;
    for _ in 0..UI_SCENE_CONTEXT_WALK_LIMIT {
        if link == 0 || link == ring_end {
            break;
        }
        let context = memory.read_u64(link)?;
        if context == ring_end {
            break;
        }
        if context != 0 && memory.read_u64(context)? == vftable {
            return Ok(Some(context));
        }
        link = memory.read_u64(link + UI_LINK_NEXT_OFFSET)?;
    }
    Ok(None)
}

/// Makes `pick` on this game's defeat screen with exactly the writes the screen's own steps make
/// for it: the cursor (step 0), the context's result and the done step (step 4).
pub fn make_pick_on_screen<M: GameMemory>(
    memory: &mut M,
    sequence: u64,
    context: u64,
    pick: DefeatPick,
) -> Result<(), MemoryError> {
    memory.write_i32(
        sequence + GAME_OVER_SEQUENCE_CURSOR_OFFSET,
        cursor_of_pick(pick),
    )?;
    memory.write_i32(
        context + GAME_OVER_CONTEXT_RESULT_OFFSET,
        game_over_result_of_pick(pick),
    )?;
    memory.write_i32(
        sequence + GAME_OVER_SEQUENCE_STATE_OFFSET,
        GAME_OVER_SEQUENCE_DONE_STATE,
    )
}

fn find_game_over_scene<M: GameMemory>(memory: &M) -> Result<Option<u64>, MemoryError> {
    let manager = memory.module_base() + u64::from(UI_MANAGER_RVA);
    let ring_end = manager + UI_MANAGER_SCENE_LIST_OFFSET;
    let mut link = memory.read_u64(manager + UI_MANAGER_FIRST_SCENE_LINK_OFFSET)?;
    for _ in 0..UI_SCENE_WALK_LIMIT {
        if link == 0 || link == ring_end {
            break;
        }
        let scene = memory.read_u64(link)?;
        if memory.read_u8(scene + UI_SCENE_ID_OFFSET)? == GAME_OVER_SCENE_ID {
            return Ok(Some(scene));
        }
        link = memory.read_u64(link + UI_LINK_NEXT_OFFSET)?;
    }
    Ok(None)
}
