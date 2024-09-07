use core::time;

use clap::Parser;
use rand::{seq::SliceRandom, Rng};
use regex::Regex;

mod cli;

fn main() {
    let config = std::fs::read_to_string("config.toml");
    let mut config = match config {
        Ok(content) => {
            let config: cli::Config = toml::from_str(&content).unwrap();
            config
        },
        Err(_) => {
            let config = cli::Config::default();
            config.save();
            config
        },
    };

    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::SetCFGPath { path } => {
            config.cfg_path = Some(path);
            config.save();
        },
        cli::Command::Randomize { } => {
            if config.cfg_path.is_none() {
                eprintln!("Error: CFG path is not set. Use `set-cfg-path` command to set it.");
                std::process::exit(1);
            }

            let re = Regex::new("alias \\x22trashtalk\\d+\\x22\\s+\\x22say\\s+(.+);\\s+alias trashtalker \\x22trashtalk\\d+\\x22").unwrap();
            let text = std::fs::read_to_string(config.cfg_path.clone().unwrap()).unwrap();
            let mut trash_talks = Vec::new();
            for cap in re.captures_iter(&text) {
                trash_talks.push(cap[1].to_string());
            }

            trash_talks.shuffle(&mut rand::thread_rng());

            let mut string_builder = String::new();
            for (i, trash_talk) in trash_talks.iter().enumerate() {
                let next = if i == trash_talks.len() - 1 {
                    0
                } else {
                    i + 1
                };
                string_builder.push_str(&format!("alias \"trashtalk{}\" \"{}; alias trashtalker \"trashtalk{}\"\"\n", i, trash_talk, next));
            }

            string_builder.push_str("alias \"trashtalker\" \"trashtalk0\";\n");
            let bind_key_re = Regex::new("bind\\s+(.+)\\s+trashtalker;").unwrap();
            let bind_key = bind_key_re.captures(&text).unwrap()[1].to_string();
            string_builder.push_str(&format!("bind {} trashtalker;", bind_key));

            std::fs::write(config.cfg_path.unwrap(), string_builder).unwrap();
        },
    }
}
