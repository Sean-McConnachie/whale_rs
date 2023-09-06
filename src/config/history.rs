use super::parse_path_buf;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigHistory {
    /// The `history_fp` is appended to the `data_dir` found in `ConfigCore`.
    #[serde(deserialize_with = "parse_path_buf")]
    pub history_fp: PathBuf,
    pub max_file_size_bytes: u64,
}

impl Default for ConfigHistory {
    fn default() -> Self {
        Self {
            history_fp: PathBuf::from("whale.history"),
            max_file_size_bytes: 256 * 1024,
        }
    }
}
