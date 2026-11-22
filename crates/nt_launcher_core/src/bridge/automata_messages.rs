use automata_proto::{Message, PathInfo};
use replicant_proto::SessionOptions;

use crate::bridge::{BridgeMessage, LauncherPayload};

impl BridgeMessage for Message {
    const LOBBY_PICKS_CHARACTERS: bool = true;
    const CARRIES_CAMPAIGN_SAVES: bool = false;
    const HOST_ALONE_TAKES_SESSION_OPTIONS: bool = true;

    fn kind_name(&self) -> &'static str {
        Message::kind_name(self)
    }

    fn path_info(rtt_micros: u32, direct: bool) -> Self {
        Message::PathInfo(PathInfo { rtt_micros, direct })
    }

    fn from_launcher(payload: LauncherPayload) -> Option<Self> {
        match payload {
            LauncherPayload::SessionOptions(options) => Some(Message::LookRule {
                any_look_allowed: options.any_look_allowed,
            }),
            _ => None,
        }
    }

    fn into_launcher(self) -> Result<LauncherPayload, Self> {
        match self {
            Message::LookRule { any_look_allowed } => {
                Ok(LauncherPayload::SessionOptions(SessionOptions {
                    any_look_allowed,
                    ..SessionOptions::default()
                }))
            }
            other => Err(other),
        }
    }
}
