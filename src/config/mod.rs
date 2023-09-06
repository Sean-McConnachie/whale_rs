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

pub fn read_or_create_all_configs() -> FullConfig {
    let config_dir = std::path::PathBuf::from(std::env::var("CONFIG_DIR").unwrap_or("./config".to_string()));

    let cfg_core: core::ConfigCore = read_or_create_config(&config_dir.join("core.toml")).unwrap_or_default();
    
    let cfg_history: history::ConfigHistory = read_or_create_config(&config_dir.join("history.toml")).unwrap_or_default();


    let cfg_theme: theme::ConfigTheme = read_or_create_config(&config_dir.join("theme.toml")).unwrap_or_default();

    let cfg_gui: gui::ConfigGUI = read_or_create_config(&config_dir.join("gui.toml")).unwrap_or_default();

    let cfg_commands: Vec<command::ConfigCommand> = command::read_commands(&config_dir.join("commands"));

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
    Ok(toml::from_str(&contents)?)
}

fn parse_path_buf<'de, D>(deserializer: D) -> Result<std::path::PathBuf, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(std::path::PathBuf::from(s))
}
