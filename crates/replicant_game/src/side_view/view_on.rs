use crate::side_view::{
    CHANGE_CAMERA_STATUS_NATIVE_RVA, MINI_MAP_DISACTIVE_NATIVE_RVA, SET_ACTION_MODE_NATIVE_RVA,
    SET_CAMERA_LEVER_MODE_NATIVE_RVA, SIDE_VIEW_MODE_ON_NATIVE_RVA, SideViewDoor,
    TERRAIN_PARTS_INVISIBLE_NATIVE_RVA, ViewArg, ViewCall, door_by_id,
};

/// A scene step after which this game's body stands in a door's place while the screen is black,
/// having never gone through the door; the host went through it earlier and has its view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneViewOn {
    pub area: &'static str,
    pub callback: &'static str,
    pub door: u16,
    pub indoor_camera_status: i32,
    pub source: &'static str,
}

pub static SCENE_VIEW_ONS: [SceneViewOn; 1] = [SceneViewOn {
    area: "A_CENTER_VILLAGE_01",
    callback: "EID_1500_a0005_010_01_End",
    door: 1,
    indoor_camera_status: 2,
    source: "run 1115j: the text scene's start put both bodies in door 1's place behind the black screen; only the host had the door's view, the guest got it 12 s after the fade in, from the door's own steps after the morning step",
}];

/// The door whose view this game takes now: its own pop of `callback` in `area` while it lacks
/// the view the host already has there.
pub fn scene_view_on(
    area: &str,
    callback: &str,
    in_side_view: bool,
    host_side_view_door: u16,
) -> Option<(&'static SceneViewOn, &'static SideViewDoor)> {
    if in_side_view {
        return None;
    }
    let scene = SCENE_VIEW_ONS
        .iter()
        .find(|scene| scene.area == area && scene.callback == callback)?;
    if host_side_view_door != scene.door {
        return None;
    }
    door_by_id(scene.area, scene.door).map(|door| (scene, door))
}

/// The game's own natives that give a door's place its side view, as the door's entry step calls
/// them, with none of the step's story work and without its later pause release.
pub fn view_on_calls(scene: &SceneViewOn, door: &SideViewDoor) -> Vec<ViewCall> {
    let call = |native_rva, args: &[ViewArg]| ViewCall {
        native_rva,
        args: args.to_vec(),
    };
    let mut calls = vec![call(SIDE_VIEW_MODE_ON_NATIVE_RVA, &[])];
    for part in door.shell_terrain_parts {
        calls.push(call(
            TERRAIN_PARTS_INVISIBLE_NATIVE_RVA,
            &[ViewArg::Text(part)],
        ));
    }
    calls.extend([
        call(
            CHANGE_CAMERA_STATUS_NATIVE_RVA,
            &[
                ViewArg::Int(scene.indoor_camera_status),
                ViewArg::Real(2.0),
                ViewArg::Int(1),
            ],
        ),
        call(
            SET_CAMERA_LEVER_MODE_NATIVE_RVA,
            &[ViewArg::Int(1), ViewArg::Int(0)],
        ),
        call(SET_ACTION_MODE_NATIVE_RVA, &[ViewArg::Int(1)]),
        call(MINI_MAP_DISACTIVE_NATIVE_RVA, &[ViewArg::Int(0)]),
    ]);
    calls
}
