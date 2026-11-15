use crate::puppet::body_info;
use replicant_proto::PartnerBody;

pub const PLAYER_MOTION_PACK_PREFIXES: [&str; 2] = ["NIER013", "KAINE015"];
pub const MOTION_SUFFIXES_HER_PACK_LACKS_WITH_THEIR_TWINS: [(&str, &str); 3] = [
    ("bioRun_lp", "run_lp"),
    ("bioRun_ed", "run_ed"),
    ("jog_lp", "run_lp"),
];
pub const MOST_DISTINCT_MISSED_MOTION_NAMES_LOGGED: usize = 48;

#[derive(Debug, Default)]
pub struct MissedMotionNames {
    seen: Vec<String>,
}

impl MissedMotionNames {
    pub const fn new() -> Self {
        Self { seen: Vec::new() }
    }

    pub fn observe(&mut self, name: &str, twin: Option<&str>) -> Option<String> {
        if name.is_empty()
            || self.seen.len() >= MOST_DISTINCT_MISSED_MOTION_NAMES_LOGGED
            || self.seen.iter().any(|seen| seen.eq_ignore_ascii_case(name))
        {
            return None;
        }
        self.seen.push(name.to_owned());
        Some(missed_motion_text(name, twin))
    }
}

pub fn swapped_body_motion_twin(name: &str) -> Option<String> {
    let after_pack_prefix = name
        .split_once('_')
        .filter(|(prefix, _)| is_player_pack_prefix(prefix))
        .map_or(name, |(_, suffix)| suffix);
    MOTION_SUFFIXES_HER_PACK_LACKS_WITH_THEIR_TWINS
        .iter()
        .find(|(missing, _)| missing.eq_ignore_ascii_case(after_pack_prefix))
        .map(|(_, twin)| format!("{}_{twin}", body_info(PartnerBody::SeedKaineE).pack_name))
}

pub fn is_player_motion_name(name: &str) -> bool {
    name.split_once('_')
        .is_some_and(|(prefix, _)| is_player_pack_prefix(prefix))
}

fn is_player_pack_prefix(prefix: &str) -> bool {
    PLAYER_MOTION_PACK_PREFIXES
        .iter()
        .any(|pack| pack.eq_ignore_ascii_case(prefix))
}

pub fn missed_motion_text(name: &str, twin: Option<&str>) -> String {
    match twin {
        Some(twin) => format!(
            "kaine motions: the game asked her body for {name}, her pack has none, it plays {twin} instead"
        ),
        None => format!(
            "kaine motions: the game asked her body for {name}, her pack has none and no twin is set, nothing plays"
        ),
    }
}
