use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shell {
    CommandPrompt,
    PowerShell,
    None,
}

/// This determines the hints that will be generated.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ArgType {
    /// Provides suggestions based on current and surrounding directories.
    Path,
    /// Provides suggestions based on executable list generated at program startup.
    Executable,
    /// Does not provide any suggestions.
    Text,
}
