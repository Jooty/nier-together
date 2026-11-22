use std::io;
use std::path::PathBuf;

use nt_launcher_core::{ALL_PROFILES, GameProfile};

use crate::app::actions::ledger_path;
use crate::config::{LauncherConfig, data_dir};
use crate::home::{GameCard, build_card};
use crate::install::{game_pids, open_url};
use crate::logs::logs_dir;
use crate::saves::coop_backup_root;

pub trait LaunchHost {
    fn open_url(&self, url: &str) -> io::Result<()>;
    fn game_pids(&self, profile: &GameProfile) -> Vec<u32>;
    fn logs_dir(&self) -> Option<PathBuf>;
    fn data_dir(&self) -> Option<PathBuf>;
    fn coop_backup_root(&self) -> Option<PathBuf>;
    fn cards(&self, config: &LauncherConfig) -> Vec<GameCard>;
}

pub struct SystemHost;

impl LaunchHost for SystemHost {
    fn open_url(&self, url: &str) -> io::Result<()> {
        open_url(url)
    }

    fn game_pids(&self, profile: &GameProfile) -> Vec<u32> {
        game_pids(profile)
    }

    fn logs_dir(&self) -> Option<PathBuf> {
        logs_dir()
    }

    fn data_dir(&self) -> Option<PathBuf> {
        data_dir()
    }

    fn coop_backup_root(&self) -> Option<PathBuf> {
        coop_backup_root()
    }

    fn cards(&self, config: &LauncherConfig) -> Vec<GameCard> {
        let ledger = ledger_path();
        ALL_PROFILES
            .iter()
            .map(|profile| build_card(profile, config, ledger.as_deref()))
            .collect()
    }
}
