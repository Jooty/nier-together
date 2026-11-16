use replicant_game::npcs::{NPC_TALK_COMMAND_IDS, refuses_talk_start, talk_busy_notice};

#[test]
fn a_talk_start_on_a_villager_the_partner_talks_to_is_refused() {
    for command in NPC_TALK_COMMAND_IDS {
        assert!(refuses_talk_start(command, true, false));
    }
}

#[test]
fn a_free_villager_or_another_command_is_never_refused() {
    for command in NPC_TALK_COMMAND_IDS {
        assert!(!refuses_talk_start(command, false, false));
    }
    assert!(!refuses_talk_start(0x14, true, false));
}

#[test]
fn the_notice_names_the_partner() {
    assert_eq!(talk_busy_notice("Ana"), "Ana is talking to them");
}

#[test]
fn a_story_steps_talk_command_is_allowed_though_the_partner_talks_to_the_villager() {
    for command in NPC_TALK_COMMAND_IDS {
        assert!(
            !refuses_talk_start(command, true, true),
            "run 1115j 01:45:41.562: the guest's own copy of the choice step was refused its villager talk, so the guest skipped straight to the text scene"
        );
    }
}
