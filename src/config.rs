use std::{env, path::PathBuf};

use crate::args::Args;

#[derive(Debug)]
pub struct Config {
    pub home: PathBuf,
    pub init_file: PathBuf,
}

impl Config {
    pub fn new(args: Args) -> Self {
        let home: PathBuf = if let Some(home) = args.home {
            home
        } else if let Some(home) = env::var_os("OXIDE_FILM_HOME") {
            PathBuf::from(home)
        } else if let Some(xdg_config_home) = env::var_os("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config_home).join("oxide-film")
        } else if cfg!(target_os = "macos") {
            PathBuf::from(env::var_os("HOME").unwrap()).join("Library/Preferences/oxide-film")
        } else {
            PathBuf::from(env::var_os("HOME").unwrap()).join(".config/oxide-film")
        };

        let init_file = if let Some(init_file) = args.init_file {
            init_file
        } else if let Some(init_file) = env::var_os("OXIDE_FILM_INIT_FILE") {
            PathBuf::from(init_file)
        } else {
            home.join("init.sh")
        };

        Self { home, init_file }
    }
}
