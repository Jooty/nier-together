use replicant_game::fight_sync::SlotState;
use replicant_game::scene_follow::{
    LocalCut, StaleCutLoad, cut_load_call, cut_to_load, stale_cut_load,
};
use replicant_game::story_relay::RelayCallGate;
use replicant_proto::{SceneCut, StoryCall};

const MANOR_PHASE: u16 = 21;
const MANOR_EVENT: &str = "EID_1100_a0075";
const MANOR_AUDIO: &str = "MA_EID_1100_a0075";
const OTHER_EVENT: &str = "EID_1000_a0030";
const HOST_SCENE_REACHED_THE_GUEST_MS: u64 = 57_694;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Path {
    SceneFollow,
    SharedFunction,
}

fn manor_cut() -> SceneCut {
    SceneCut {
        event: MANOR_EVENT.to_owned(),
        audio: MANOR_AUDIO.to_owned(),
    }
}

fn no_cut() -> LocalCut {
    LocalCut {
        loaded: false,
        event: None,
    }
}

fn loaded(event: &str) -> LocalCut {
    LocalCut {
        loaded: true,
        event: Some(event.to_owned()),
    }
}

fn tick_as_the_hook_does(
    gate: &mut RelayCallGate<Path>,
    slot: SlotState,
    local: &LocalCut,
    local_in_scene: bool,
) -> (Vec<StaleCutLoad>, Option<StoryCall>) {
    let dropped = gate
        .withdraw_waiting_if(|waiting| {
            waiting.then == Path::SceneFollow
                && stale_cut_load(&waiting.call, local, local_in_scene).is_some()
        })
        .into_iter()
        .filter_map(|waiting| stale_cut_load(&waiting.call, local, local_in_scene))
        .collect();
    let due = gate.tick(slot, false, |_| false).map(|due| due.call);
    (dropped, due)
}

#[test]
fn the_cut_load_the_host_scene_asked_for_is_dropped_once_the_guests_own_scene_runs() {
    // Run 2026-11-16-124144: the host's scene reached the guest while its stage start held the
    // slot, the guest's own scripts loaded the cut and started the scene, then the slot freed.
    let mut gate = RelayCallGate::<Path>::new();
    let cut = cut_to_load(&no_cut(), Some(&manor_cut())).expect("the guest had no cut");
    gate.submit(
        cut_load_call(MANOR_PHASE, &cut),
        Path::SceneFollow,
        HOST_SCENE_REACHED_THE_GUEST_MS,
    );
    let (dropped, due) = tick_as_the_hook_does(&mut gate, SlotState::Busy, &no_cut(), false);
    assert_eq!((dropped, due), (Vec::new(), None));
    // 58.027: the guest's own scene starts with its own copy of the cut loaded.
    let (dropped, due) =
        tick_as_the_hook_does(&mut gate, SlotState::Busy, &loaded(MANOR_EVENT), true);
    assert_eq!(dropped, vec![StaleCutLoad::OwnSceneRuns]);
    assert_eq!(due, None);
    // 58.881: the slot frees inside the scene; before the fix the load went here.
    let (dropped, due) =
        tick_as_the_hook_does(&mut gate, SlotState::Idle, &loaded(MANOR_EVENT), true);
    assert_eq!(
        (dropped, due),
        (Vec::new(), None),
        "no cut load may reach the guest's queue inside its own scene"
    );
}

#[test]
fn a_cut_load_whose_cut_the_guests_scripts_loaded_meanwhile_is_dropped() {
    let mut gate = RelayCallGate::<Path>::new();
    gate.submit(
        cut_load_call(MANOR_PHASE, &manor_cut()),
        Path::SceneFollow,
        HOST_SCENE_REACHED_THE_GUEST_MS,
    );
    let (dropped, due) =
        tick_as_the_hook_does(&mut gate, SlotState::Idle, &loaded(MANOR_EVENT), false);
    assert_eq!(dropped, vec![StaleCutLoad::CutLoadedHere]);
    assert_eq!(due, None);
}

#[test]
fn a_cut_load_still_needed_outside_a_scene_goes_when_the_slot_frees() {
    let mut gate = RelayCallGate::<Path>::new();
    let load = cut_load_call(MANOR_PHASE, &manor_cut());
    gate.submit(
        load.clone(),
        Path::SceneFollow,
        HOST_SCENE_REACHED_THE_GUEST_MS,
    );
    let (dropped, due) = tick_as_the_hook_does(&mut gate, SlotState::Busy, &no_cut(), false);
    assert_eq!((dropped, due), (Vec::new(), None));
    let (dropped, due) =
        tick_as_the_hook_does(&mut gate, SlotState::Idle, &loaded(OTHER_EVENT), false);
    assert_eq!((dropped, due), (Vec::new(), Some(load)));
}

#[test]
fn only_the_scene_follows_cut_load_is_judged_stale() {
    let mut gate = RelayCallGate::<Path>::new();
    let start = StoryCall {
        phase: MANOR_PHASE,
        pack: 0,
        function: format!("{MANOR_EVENT}_Start"),
        args: Vec::new(),
    };
    assert_eq!(stale_cut_load(&start, &loaded(MANOR_EVENT), true), None);
    let shared_load = cut_load_call(MANOR_PHASE, &manor_cut());
    gate.submit(
        start.clone(),
        Path::SceneFollow,
        HOST_SCENE_REACHED_THE_GUEST_MS,
    );
    gate.submit(
        shared_load.clone(),
        Path::SharedFunction,
        HOST_SCENE_REACHED_THE_GUEST_MS,
    );
    let (dropped, due) =
        tick_as_the_hook_does(&mut gate, SlotState::Idle, &loaded(MANOR_EVENT), true);
    assert_eq!((dropped, due), (Vec::new(), Some(start)));
    assert_eq!(gate.waiting_len(), 1);
}
