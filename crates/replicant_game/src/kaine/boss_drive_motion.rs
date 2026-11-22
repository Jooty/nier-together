use crate::kaine::is_player_motion_name;

pub const MOST_DISTINCT_BOSS_DRIVE_MISSES_LOGGED: usize = 200;

pub const GUEST_MOTION_KEYWORDS_TO_HER_MOTIONS: &[(&str, &str)] = &[
    ("dmgs", "kaine010_dm_damages^r_01"),
    ("dmgm", "kaine010_dm_damagem^r_01"),
    ("dmgl", "kaine010_dm_damagel^f_01"),
    ("springup", "kaine010_dm_damagel^f^standup_01"),
    ("blow", "kaine010_dm_blow^f_01"),
    ("guardwalk", "kaine010_df_guardwalk^f^lo_01"),
    ("guard", "kaine010_df_guardstay^lo_01"),
    ("atks_5", "kaine010_at_sword^c06_01"),
    ("atks_4", "kaine010_at_sword^c05_01"),
    ("atks_3", "kaine010_at_sword^c04_01"),
    ("atks_2", "kaine010_at_sword^c03_01"),
    ("atks_1", "kaine010_at_sword^c02_01"),
    ("atks", "kaine010_at_sword^c01_01"),
    ("atkh", "kaine010_at_sword^t01_01"),
    ("atk", "kaine010_at_sword^c01_01"),
    ("mag", "kaine010_at_magic^c01_01"),
    ("dodge_b", "kaine010_01_escape^b_01"),
    ("dodge_l", "kaine010_01_escape^l_01"),
    ("dodge_r", "kaine010_01_escape^r_01"),
    ("dodge", "kaine010_01_escape^f_01"),
    ("dashfromb", "kaine010_01_escape^b_01"),
    ("dashfroml", "kaine010_01_escape^l_01"),
    ("dashfromr", "kaine010_01_escape^r_01"),
    ("dashfromf", "kaine010_01_escape^f_01"),
    ("dash", "kaine010_01_escape^f_01"),
    ("jump_ed", "kaine010_01_jump^e_01"),
    ("jump", "kaine010_01_jump^01_01"),
    ("land", "kaine010_02_land_01"),
    ("fall", "kaine010_01_fall^lo_01"),
    ("run_ed", "kaine010_01_dash^e_01"),
    ("run", "kaine010_01_dash^lo_01"),
    ("jog", "kaine010_01_dash^lo_01"),
    ("walk", "kaine010_01_walk^lo_01"),
    ("turn", "kaine010_01_turn_01"),
    ("idle", "kaine010_01_stay^lo_01"),
    ("wait", "kaine010_01_stay^lo_01"),
    ("stay", "kaine010_01_stay^lo_01"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HerMotionChoice {
    Play { wanted: &'static str, index: i32 },
    NotInHerPack { wanted: &'static str },
    NoFamily,
}

#[derive(Debug, Default)]
pub struct BossDriveMisses {
    seen: Vec<String>,
}

impl BossDriveMisses {
    pub const fn new() -> Self {
        Self { seen: Vec::new() }
    }

    pub fn observe(&mut self, guest_motion_name: &str, choice: HerMotionChoice) -> Option<String> {
        let text = match choice {
            HerMotionChoice::Play { .. } => return None,
            HerMotionChoice::NotInHerPack { wanted } => format!(
                "boss drive: the guest's {guest_motion_name} maps to {wanted}, which her pack does not hold; she keeps her last motion"
            ),
            HerMotionChoice::NoFamily => format!(
                "boss drive: the guest's {guest_motion_name} has no twin in her pack; she keeps her last motion"
            ),
        };
        if self.seen.len() >= MOST_DISTINCT_BOSS_DRIVE_MISSES_LOGGED
            || self
                .seen
                .iter()
                .any(|seen| seen.eq_ignore_ascii_case(guest_motion_name))
        {
            return None;
        }
        self.seen.push(guest_motion_name.to_owned());
        Some(text)
    }
}

pub fn her_motion_for(
    guest_motion_name: &str,
    mut index_of: impl FnMut(&'static str) -> Option<i32>,
) -> HerMotionChoice {
    let lowered = guest_motion_name.to_ascii_lowercase();
    let without_pack = if is_player_motion_name(&lowered) {
        lowered
            .split_once('_')
            .map_or(lowered.as_str(), |(_, rest)| rest)
    } else {
        lowered.as_str()
    };
    let Some(&(_, wanted)) = GUEST_MOTION_KEYWORDS_TO_HER_MOTIONS
        .iter()
        .find(|(keyword, _)| without_pack.contains(*keyword))
    else {
        return HerMotionChoice::NoFamily;
    };
    match index_of(wanted) {
        Some(index) => HerMotionChoice::Play { wanted, index },
        None => HerMotionChoice::NotInHerPack { wanted },
    }
}

pub fn scaled_frame(guest_frame: f32, guest_frame_count: i32, her_frame_count: i32) -> f32 {
    if guest_frame_count <= 1 || her_frame_count <= 1 || !guest_frame.is_finite() {
        return 0.0;
    }
    let guest_last = (guest_frame_count - 1) as f32;
    let her_last = (her_frame_count - 1) as f32;
    (guest_frame.clamp(0.0, guest_last) / guest_last) * her_last
}
