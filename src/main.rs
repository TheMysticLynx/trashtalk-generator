use core::time;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use clap::Parser;
use rand::{seq::SliceRandom, Rng};
use regex::Regex;
use uuid::Uuid;

mod cli;

fn main() {
    let config = std::fs::read_to_string("config.toml");
    let mut config = match config {
        Ok(content) => {
            let config: cli::Config = toml::from_str(&content).unwrap();
            config
        }
        Err(_) => {
            let config = cli::Config::default();
            config.save();
            config
        }
    };

    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::SetCFGPath { path } => {
            config.cfg_path = Some(path);
            config.save();
        }
        cli::Command::Randomize {} => {
            for keybind in config.keybinds.as_ref().unwrap().values() {
                let trashtalks = get_trashtalk_from_file(&keybind.trashtalk_file_path);
                if trashtalks.is_empty() {
                    println!(
                        "Warning: No trashtalks found in file '{}'. Skipping keybind '{}'.",
                        keybind.trashtalk_file_path, keybind.key
                    );
                    continue;
                }

                let bind_string = generate_cfg_string(&trashtalks, &keybind.key, &keybind.uuid);
                let cfg_path = config.cfg_path.as_ref().unwrap();
                let bind_file_path =
                    Path::new(cfg_path).join(format!("trashtalk_{}.cfg", keybind.uuid));

                std::fs::write(bind_file_path, bind_string).unwrap();
                println!(
                    "Keybind '{}' randomized with trashtalks from '{}'",
                    keybind.key, keybind.trashtalk_file_path
                );
            }
        }
        cli::Command::AddKeybind {
            key,
            trashtalk_file_path,
        } => {
            if config.keybinds.is_none() {
                config.keybinds = Some(HashMap::new());
            }

            if config.keybinds.as_ref().unwrap().contains_key(&key) {
                println!(
                    "Warning: Keybind for '{}' already exists. Overwriting.",
                    key
                );
                config.keybinds.as_mut().unwrap().remove(&key);
            }

            let mut uuid;
            loop {
                uuid = Uuid::new_v4().to_string();
                // check if the UUID is already used
                if !config
                    .keybinds
                    .as_ref()
                    .unwrap()
                    .values()
                    .any(|k| k.uuid == uuid)
                {
                    break;
                }
            }

            config.keybinds.as_mut().unwrap().insert(
                key.clone(),
                cli::Keybind {
                    key: key.clone(),
                    trashtalk_file_path: trashtalk_file_path.clone(),
                    uuid,
                },
            );
            config.save();
            println!(
                "Keybind for '{}' added with UUID: {}",
                key,
                config.keybinds.as_ref().unwrap().get(&key).unwrap().uuid
            );
        }
        cli::Command::RemoveKeybind { key } => {
            if let Some(keybinds) = &mut config.keybinds {
                if keybinds.remove(&key).is_some() {
                    println!("Keybind for '{}' removed.", key);
                } else {
                    println!("No keybind found for '{}'.", key);
                }
            } else {
                println!("No keybinds found.");
            }
            config.save();
        }
        cli::Command::ListKeybinds {} => {
            if let Some(keybinds) = &config.keybinds {
                if keybinds.is_empty() {
                    println!("No keybinds found.");
                } else {
                    for (key, bind) in keybinds {
                        println!(
                            "Key: {}, Trashtalk File: {}, UUID: {}",
                            key, bind.trashtalk_file_path, bind.uuid
                        );
                    }
                }
            } else {
                println!("No keybinds found.");
            }
        }
        cli::Command::Init {} => {
            // grab contents from the cfg file
            if config.cfg_path.is_none() {
                eprintln!("Error: CFG path is not set. Use `set-cfg-path` command to set it.");
                std::process::exit(1);
            }
            let cfg_path = config.cfg_path.as_ref().unwrap();

            // get or create the autoexec.cfg file
            let autoexec_path = Path::new(cfg_path).join("autoexec.cfg");
            if !autoexec_path.exists() {
                std::fs::write(&autoexec_path, "").unwrap();
            }

            // our custom section starts with a comment saying "Custom Trashtalks"
            let custom_section_start = "// Custom Trashtalks";
            let custom_section_end = "// End of Custom Trashtalks";

            // find the start and end of the custom section and then remove it
            let content = std::fs::read_to_string(&autoexec_path).unwrap();
            let mut new_content = String::new();
            let mut in_custom_section = false;
            for line in content.lines() {
                if line.contains(custom_section_start) {
                    in_custom_section = true;
                    continue; // skip the start line
                }
                if line.contains(custom_section_end) {
                    in_custom_section = false;
                    continue; // skip the end line
                }
                if !in_custom_section {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }

            // get all uuids from the trashtalk files
            let mut uuids = HashSet::new();
            if let Some(keybinds) = &config.keybinds {
                for bind in keybinds.values() {
                    uuids.insert(bind.uuid.clone());
                }
            }

            // add the custom section to the end of the file
            new_content.push_str("// Custom Trashtalks\n");
            for uuid in &uuids {
                let trashtalk_file_path =
                    Path::new(cfg_path).join(format!("trashtalk_{}.cfg", uuid));
                if !trashtalk_file_path.exists() {
                    eprintln!(
                        "Trashtalk file '{}' does not exist. Creating.",
                        trashtalk_file_path.display()
                    );
                    std::fs::write(&trashtalk_file_path, "").unwrap();
                }

                new_content.push_str(&format!("exec \"{}\"\n", trashtalk_file_path.display()));
            }
            new_content.push_str("// End of Custom Trashtalks\n");
            std::fs::write(&autoexec_path, new_content).unwrap();
        }
        cli::Command::ExtractTrashtalks {
            output_file,
            infput_cfg,
        } => {
            if !Path::new(&infput_cfg).exists() {
                eprintln!("Input CFG file does not exist: {}", infput_cfg);
                std::process::exit(1);
            }

            let trashtalks = get_trashtalk_from_cfg(&infput_cfg);
            if trashtalks.is_empty() {
                println!("No trashtalks found in the input CFG file.");
            } else {
                std::fs::write(output_file.clone(), trashtalks.join("\n")).unwrap();
                println!(
                    "Extracted {} trashtalks to '{}'.",
                    trashtalks.len(),
                    output_file
                );
            }
        },
    }
}

fn get_trashtalk_from_cfg(file_path: &str) -> Vec<String> {
    let content = std::fs::read_to_string(file_path).unwrap();
    let re = Regex::new("say\\s+(.+);\\s+alias").unwrap();
    re.captures_iter(&content)
        .map(|cap| cap[1].to_string())
        .collect()
}

fn generate_cfg_string(trashtalks: &[String], bind_key: &str, uuid: &str) -> String {
    let mut string_builder = String::new();
    for (i, trashtalk) in trashtalks.iter().enumerate() {
        let next = if i == trashtalks.len() - 1 { 0 } else { i + 1 };
        string_builder.push_str(&format!(
            "alias \"trashtalk{}{}\" \"say {}; alias trashtalker \"trashtalk{}{}\"\n",
            i, uuid, trashtalk, uuid, next
        ));
    }
    string_builder.push_str(&format!("alias \"trashtalker{}\" \"trashtalk0\";\n", uuid));
    string_builder.push_str(&format!("bind {} trashtalker{};", bind_key, uuid));
    string_builder
}

fn get_trashtalk_from_file(file_path: &str) -> Vec<String> {
    let content = std::fs::read_to_string(file_path).unwrap();
    // filter out empty lines and comments
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                None
            } else {
                Some(line.to_string())
            }
        })
        .collect()
}
