extern crate toml;

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::{fs, path};

// Multiple config files:
//  - Whale.toml
//  - Theme.toml
//  - Commands/*.toml
//  - History.toml
//  - Views.toml


// ================================================ Command Config
#[derive(Debug, Serialize, Deserialize)]
pub enum ArgType {
    Path,
    Executable,
    Text,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    #[serde(skip, default)]
    pub exe_name: String,
    pub exe_to: String,

    pub execute_before: Option<String>,
    pub execute_after: Option<String>,

    #[serde(rename = "arg")]
    pub args: Vec<Arg>,
    #[serde(rename = "flag")]
    pub flags: Vec<Flag>,
    #[serde(rename = "arg_flag")]
    pub arg_flags: Vec<ArgFlag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Arg {
    pub arg_type: ArgType,
    pub arg_hint: Option<String>,
    pub arg_pos: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Flag {
    pub flag_name: String,
    pub flag_to: Option<String>,

    pub execute_before: Option<String>,
    pub execute_after: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArgFlag {
    pub flag_name: String,
    pub flag_to: Option<String>,

    pub arg_type: ArgType,
    pub arg_hint: Option<String>,
}
// ================================================ Theme Config
#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub error: SuperStyle,
    pub console: SuperStyle,
    pub executable: SuperStyle,
    pub path: SuperStyle,
    pub flag: SuperStyle,
    pub text: SuperStyle,
    pub other: SuperStyle,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SuperStyle {
    pub normal: Style,
    pub highlighted: Style,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Style {
    modifiers: Vec<Formatter>,
    foreground: Color,
    background: Option<Color>,

    #[serde(skip)]
    pub escape_sequence: String,
}
#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Formatter {
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hidden,
    Strike,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}
// ================================================ History Config DONE
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryConfig {
    #[serde(deserialize_with = "parse_path_buf")]
    pub file_path: path::PathBuf,
    pub max_file_size_bytes: u64,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        let config_dir = path::PathBuf::from("config");
        Self {
            file_path: config_dir.join(path::PathBuf::from("whale.history")),
            max_file_size_bytes: 256 * 1024,
        }
    }
}

// ================================================ Views Config
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ViewsConfig {
    #[serde(rename = "command_output")]
    pub basic_command_output: crate::views::ViewCommandOutputConfig,

    #[serde(rename = "list_column")]
    pub list_column: crate::views::ViewListColumnConfig,
}

// ================================================= FUNCTIONs
pub fn read_or_create_config<P: AsRef<path::Path>, T: for<'de> Deserialize<'de> + Default + Serialize>(
    path: P,
) -> anyhow::Result<T> {
    let path = path.as_ref();
    if !path.exists() {
        let config = T::default();
        let toml_str = toml::to_string_pretty(&config)?;
        let mut file = fs::File::create(&path)?;
        file.write_all(toml_str.as_bytes())?;
        return Ok(config);
    };
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(toml::from_str(&contents)?)
}

fn parse_path_buf<'de, D>(deserializer: D) -> Result<path::PathBuf, D::Error>
    where
        D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(std::path::PathBuf::from(s))
}
