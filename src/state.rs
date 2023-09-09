use std::path;
use crate::config;

pub struct ProgramState {
    config: config::FullConfig,
    current_working_directory: path::PathBuf,
}

impl ProgramState {
    pub fn init(config: config::FullConfig, current_working_directory: path::PathBuf) -> Self {
        Self {
            config,
            current_working_directory
        }
    }
}