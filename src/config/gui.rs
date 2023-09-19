use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigGUI {
    pub table: ConfigTableGUI,
}

impl Default for ConfigGUI {
    fn default() -> Self {
        Self {
            table: ConfigTableGUI::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigTableGUI {
    pub max_field_len: u16,
}

impl Default for ConfigTableGUI {
    fn default() -> Self {
        Self {
            max_field_len: 20,
        }
    }
}