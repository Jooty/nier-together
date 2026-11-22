use nt_hook::hook_env::read_env;
use nt_hook::net::{game_folder, read_config_text};

use crate::raw_mouse_setting::{RAW_MOUSE_ENV, RAW_MOUSE_KEY};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMouseChoice {
    pub on: bool,
    pub log_line: String,
}

pub fn choose_raw_mouse(env_value: Option<String>, config_text: Option<&str>) -> RawMouseChoice {
    let from_config = config_text
        .and_then(|text| toml::from_str::<toml::Table>(text).ok())
        .and_then(|table| match table.get(RAW_MOUSE_KEY)? {
            toml::Value::String(text) => Some(text.clone()),
            toml::Value::Boolean(flag) => Some(if *flag { "on" } else { "off" }.to_owned()),
            _ => None,
        });
    let given = env_value.or(from_config);
    match given.as_deref().map(str::trim) {
        None => RawMouseChoice {
            on: false,
            log_line: "raw mouse: not given, off".to_owned(),
        },
        Some("on" | "1" | "true") => RawMouseChoice {
            on: true,
            log_line: "raw mouse: on".to_owned(),
        },
        Some("off" | "0" | "false") => RawMouseChoice {
            on: false,
            log_line: "raw mouse: off".to_owned(),
        },
        Some(text) => RawMouseChoice {
            on: false,
            log_line: format!("raw mouse: unknown value {text:?}, using off"),
        },
    }
}

pub fn raw_mouse_from_launcher() -> RawMouseChoice {
    let config_text = game_folder()
        .and_then(|folder| read_config_text(&folder).ok())
        .flatten();
    choose_raw_mouse(read_env(RAW_MOUSE_ENV), config_text.as_deref())
}
