use std::path;
use crate::config;

#[derive(Debug)]
pub struct ProgramState {
    pub config: config::FullConfig,
    pub current_working_directory: path::PathBuf,
}

impl ProgramState {
    pub fn init(config: config::FullConfig, current_working_directory: path::PathBuf) -> Self {
        Self {
            config,
            current_working_directory
        }
    }
}
