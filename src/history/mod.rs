use serde::{Deserialize, Serialize};

pub mod ux_layer;
pub mod data_layer;

const BUFFER_SIZE: usize = 16384;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    timestamp: u64,
    command: String,
}

impl HistoryEntry {
    fn new(timestamp: u64, command: String) -> Self {
        Self {
            timestamp,
            command,
        }
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn to_line(&self) -> String {
        format!("{} {}\n", self.timestamp, self.command)
    }

    fn from_line(line: &str, line_number: u32) -> anyhow::Result<Self> {
        for (i, c) in line.chars().enumerate() {
            if c == ' ' {
                let timestamp = line[..i].parse::<u64>().map_err(|e| {
                    anyhow::anyhow!("Failed to parse timestamp from history file: {}", e)
                })?;
                if i + 1 >= line.len() {
                    return Err(anyhow::anyhow!(
                        "Index out of bounds for parsing historical entry on line: {}",
                        line_number
                    ));
                }
                return Ok(Self::new(timestamp, line[i + 1..].to_string()));
            }
        }

        Err(anyhow::anyhow!("Empty line on line: {}", line_number))
    }
}
