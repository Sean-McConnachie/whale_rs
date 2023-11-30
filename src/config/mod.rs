extern crate dotenv;

pub mod command;
pub mod core;
pub mod gui;
pub mod history;
pub mod theme;

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Debug)]
pub struct FullConfig {
    pub core: core::ConfigCore,
    pub history: history::ConfigHistory,
    pub theme: theme::ConfigTheme,
    pub gui: gui::ConfigGUI,
    pub commands: Vec<command::ConfigCommand>,
}

impl Default for FullConfig {
    fn default() -> Self {
        Self {
            core: core::ConfigCore::default(),
            history: history::ConfigHistory::default(),
            theme: theme::ConfigTheme::default(),
            gui: gui::ConfigGUI::default(),
            commands: vec![],
        }
    }
}

pub fn read_or_create_all_configs() -> FullConfig {
    let config_dir = {
        let exe_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        std::env::set_current_dir(exe_dir).unwrap();

        match dotenv::dotenv() {
            Ok(_) => (),
            Err(e) => panic!("Error loading .env file: {}", e),
        }
        let env_config_dir = std::env::var("CONFIG_DIR").unwrap_or("./config".to_string());
        std::path::PathBuf::from(env_config_dir)
    };

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).unwrap();
    }

    let mut cfg_core: core::ConfigCore =
        read_or_create_config(&config_dir.join("core.toml")).unwrap_or_default();
    cfg_core.config_dir = config_dir.clone();

    if !cfg_core.data_dir.exists() {
        std::fs::create_dir_all(&cfg_core.data_dir).unwrap();
    }

    let cfg_history: history::ConfigHistory =
        read_or_create_config(&config_dir.join("history.toml")).unwrap_or_default();

    let mut cfg_theme: theme::ConfigTheme =
        read_or_create_config(&config_dir.join("theme.toml")).unwrap_or_default();
    cfg_theme.generate_escape_sequences();

    let cfg_gui: gui::ConfigGUI =
        read_or_create_config(&config_dir.join("gui.toml")).unwrap_or_default();

    let cfg_commands: Vec<command::ConfigCommand> =
        command::read_commands(&config_dir.join("commands"));

    FullConfig {
        core: cfg_core,
        history: cfg_history,
        theme: cfg_theme,
        gui: cfg_gui,
        commands: cfg_commands,
    }
}

pub fn read_or_create_config<
    P: AsRef<std::path::Path>,
    T: for<'de> Deserialize<'de> + Default + Serialize,
>(
    path: P,
) -> anyhow::Result<T> {
    let path = path.as_ref();
    if !path.exists() {
        let config = T::default();
        let toml_str = toml::to_string_pretty(&config)?;
        let mut file = std::fs::File::create(&path)?;
        file.write_all(toml_str.as_bytes())?;
        return Ok(config);
    };
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(toml::from_str(&contents).unwrap())
}

fn parse_path_buf<'de, D>(deserializer: D) -> Result<std::path::PathBuf, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(std::path::PathBuf::from(s))
}
