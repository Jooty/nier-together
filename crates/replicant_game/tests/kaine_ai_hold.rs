use replicant_game::kaine::{HerAiHoldReading, format_held_ticks, her_ai_is_held};

fn kept_off() -> HerAiHoldReading {
    HerAiHoldReading {
        kept_off: true,
        her_script_mode: false,
        event_scene: false,
    }
}

#[test]
fn an_active_tick_outside_her_script_mode_and_any_scene_is_held() {
    // run-1115j host run1, 01:52:54.038 to 01:52:57.048: active byte 1 throughout the call lend after
    // StartStage, no scene, her script mode 0: three seconds of her AI on build 55869146
    assert!(her_ai_is_held(kept_off()));
}

#[test]
fn her_script_mode_runs_as_the_game_wants() {
    assert!(!her_ai_is_held(HerAiHoldReading {
        her_script_mode: true,
        ..kept_off()
    }));
}

#[test]
fn an_event_scene_runs_as_the_game_wants() {
    assert!(!her_ai_is_held(HerAiHoldReading {
        event_scene: true,
        ..kept_off()
    }));
}

#[test]
fn nothing_is_held_while_the_rule_does_not_keep_her_off() {
    assert!(!her_ai_is_held(HerAiHoldReading {
        kept_off: false,
        ..kept_off()
    }));
}

#[test]
fn the_held_count_reads_as_a_sentence() {
    assert_eq!(format_held_ticks(0), "her update ran as the game's");
    assert_eq!(
        format_held_ticks(180),
        "her update was held 180 ticks (active with her AI off)"
    );
}
