use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigGUI {
    pub table: ConfigTableGUI,
    pub dropdown: ConfigDropdownGUI,
}

impl Default for ConfigGUI {
    fn default() -> Self {
        Self {
            table: ConfigTableGUI::default(),
            dropdown: ConfigDropdownGUI::default(),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDropdownGUI {
    pub max_rows: u16,
}

impl Default for ConfigDropdownGUI {
    fn default() -> Self {
        Self {
            max_rows: 10,
        }
    }
}
