use std::path;
use crate::{config, enums};

#[derive(Debug)]
pub struct ProgramState {
    pub config: config::FullConfig,
    pub current_working_directory: path::PathBuf,
    pub current_shell: enums::Shell,
}

impl ProgramState {
    pub fn init(
        config: config::FullConfig,
        current_working_directory: path::PathBuf,
        current_shell: enums::Shell,
    ) -> Self {
        Self {
            config,
            current_working_directory,
            current_shell,
        }
    }
}
