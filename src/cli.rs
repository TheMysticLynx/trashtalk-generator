use std::{collections::HashMap, path::Path};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command
}

#[derive(Debug, Subcommand)]
pub enum Command {
    SetCFGPath {
        path: String,
    },
    Init {

    },
    Randomize {
    },
    AddKeybind {
        key: String,
        trashtalk_file_path: String,
    },
    RemoveKeybind {
        key: String,
    },
    ListKeybinds {
    },
    ExtractTrashtalks {
        output_file: String,
        infput_cfg: String,
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub cfg_path: Option<String>,
    pub keybinds: Option<HashMap<String, Keybind>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Keybind {
    pub key: String,
    pub trashtalk_file_path: String,
    pub uuid: String,
}


impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save(&self) {
        let config_str = toml::to_string(&self).unwrap();
        std::fs::write("config.toml", config_str).unwrap();
    }
}