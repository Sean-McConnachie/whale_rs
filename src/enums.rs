use std::process;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shell {
    CommandPrompt,
    PowerShell,
    None,
}

#[cfg(target_os = "windows")]
impl Shell {
    pub fn to_exec(&self) -> process::Command {
        match self {
            Shell::CommandPrompt => {
                let mut c = process::Command::new("cmd");
                c.arg("/c");
                c
            }
            Shell::PowerShell => {
                let mut c = process::Command::new("powershell");
                c.arg("-Command");
                c
            }
            Shell::None => {
                let mut c = process::Command::new("");
                c
            }
        }
    }
}

#[cfg(target_os = "windows")]
impl Default for Shell {
    fn default() -> Self {
        Self::CommandPrompt
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    None,
}

#[cfg(target_os = "linux")]
impl Shell {
    pub fn to_exec(&self) -> process::Command {
        match self {
            Shell::Bash => {
                let mut c = process::Command::new("bash");
                c.arg("-c");
                c
            }
            Shell::Zsh => {
                let mut c = process::Command::new("zsh");
                c.arg("-c");
                c
            }
            Shell::Fish => {
                let mut c = process::Command::new("fish");
                c.arg("-c");
                c
            }
            Shell::None => {
                let c = process::Command::new("");
                c
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Default for Shell {
    fn default() -> Self {
        Self::Bash
    }
}

/// This determines the hints that will be generated.
#[derive(Debug, PartialEq, Clone, Default, Copy, Serialize, Deserialize)]
pub enum ArgType {
    /// Provides suggestions based on current and surrounding directories.
    Path,
    /// Provides suggestions based on executable list generated at program startup.
    Executable,
    /// Does not provide any suggestions.
    #[default]
    Text,
}
