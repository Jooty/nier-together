use replicant_game::stay_together::FollowRole;
use replicant_game::story_calls::{SceneScope, StoryCallKind, name_hash, story_call_kind_by_name};
use replicant_game::story_relay::{CallKey, FireAction, FireFacts, StoryRelayLedger};

const RED_BAG_CALL: &str = "Red_Bag_Get";
const RED_BAG_PACK: &str = "A_SOUTH_FIELD_01";
const RED_BAG_FUNCTION_HASH_IN_THE_LOG: u64 = 0xbd06_93b3_e12f_5f7a;
const RED_BAG_ARGS_HASH_IN_THE_LOG: u64 = 0xcbf2_9ce4_8422_2325;

fn host_facts(partner_here: bool) -> FireFacts {
    FireFacts {
        enabled: true,
        connected: true,
        role: FollowRole::Host,
        local_in_play: true,
        partner_in_same_phase: true,
        kind: Some(story_call_kind_by_name(RED_BAG_PACK, RED_BAG_CALL)),
        scope: SceneScope::of(RED_BAG_CALL),
        partner_here,
        owned_elsewhere: false,
        args_relayable: true,
        partner_too_far: !partner_here,
        can_gather_partner: !partner_here,
        each_run_counts: false,
        partners_copy: false,
    }
}

fn fire(facts: &FireFacts) -> FireAction {
    let key = CallKey {
        function: RED_BAG_FUNCTION_HASH_IN_THE_LOG,
        args: RED_BAG_ARGS_HASH_IN_THE_LOG,
    };
    StoryRelayLedger::new().on_fire(0x1000, facts, key, 0)
}

#[test]
fn the_red_bag_call_is_the_one_the_log_shows() {
    assert_eq!(name_hash(RED_BAG_CALL), RED_BAG_FUNCTION_HASH_IN_THE_LOG);
    assert_eq!(
        story_call_kind_by_name(RED_BAG_PACK, RED_BAG_CALL),
        StoryCallKind::Story
    );
}

#[test]
fn the_red_bag_call_is_a_shared_scene() {
    assert_eq!(SceneScope::of(RED_BAG_CALL), SceneScope::SharedScene);
}

#[test]
fn a_far_guest_is_pulled_before_the_red_bag_talk_plays() {
    assert_eq!(fire(&host_facts(false)), FireAction::HoldForGatheredPartner);
}

#[test]
fn a_guest_beside_the_host_sees_the_red_bag_talk_without_a_wait() {
    assert_eq!(fire(&host_facts(true)), FireAction::RunAndAnnounce);
}

#[test]
fn a_guest_in_another_area_leaves_the_red_bag_talk_to_the_host() {
    let facts = FireFacts {
        partner_in_same_phase: false,
        partner_too_far: false,
        can_gather_partner: false,
        ..host_facts(false)
    };
    assert_ne!(fire(&facts), FireAction::HoldForGatheredPartner);
}
