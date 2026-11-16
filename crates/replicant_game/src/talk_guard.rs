mod important_talk_bases;
mod important_talk_ids;
mod press;
mod registry;
mod side_view_report;

pub use important_talk_bases::IMPORTANT_TALK_BASES;
pub use important_talk_ids::IMPORTANT_TALK_IDS;
pub use press::{
    PartnerSideView, PressAsk, PressVerdict, important_base_of_talk_id, is_important_talk,
    judge_press, partner_inside, talk_needs_the_partner_in_the_house,
};
pub use registry::TalkRegistry;
pub use side_view_report::SideViewReporter;

pub const TALK_REGISTRATION_MODE_COUNT: usize = 2;
pub const TALK_IDS_THAT_NEED_THE_PARTNER_IN_THE_HOUSE: [i32; 2] = [1321, 1322];
pub const SIDE_VIEW_STATE_RESENT_EVERY_MS: u64 = 1_000;
