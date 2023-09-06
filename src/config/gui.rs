use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigGUI {
}

impl Default for ConfigGUI {
    fn default() -> Self {
        Self {}
    }
}
