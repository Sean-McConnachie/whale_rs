use super::parse_path_buf;
use crate::enums::Shell;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigCore {
    pub default_shell: Shell,

    /// Retrieved from environment variable CONFIG_DIR
    #[serde(skip_serializing)]
    #[serde(default)]
    pub config_dir: PathBuf,

    #[serde(deserialize_with = "parse_path_buf")]
    pub data_dir: PathBuf,
}

impl Default for ConfigCore {
    fn default() -> Self {
        let config_dir = PathBuf::from("");
        let data_dir = PathBuf::from("data");
        Self {
            default_shell: Shell::default(),
            config_dir,
            data_dir,
        }
    }
}
