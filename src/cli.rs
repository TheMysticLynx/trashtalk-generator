use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

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
    Randomize {
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub cfg_path: Option<String>,
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