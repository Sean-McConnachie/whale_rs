use std::{fs, io::{self, Write}, path, time};
use std::cell::RefCell;
use std::rc::Rc;
use crate::config::history;
use crate::state;

#[derive(Debug)]
struct HistoryEntry {
    timestamp: u64,
    line_number: u32,
    command: String,
}

impl HistoryEntry {
    fn new(timestamp: u64, line_number: u32, command: String) -> Self {
        Self {
            timestamp,
            line_number,
            command,
        }
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
                return Ok(Self::new(timestamp, line_number, line[i + 1..].to_string()));
            }
        }

        Err(anyhow::anyhow!("Empty line on line: {}", line_number))
    }
}

#[derive(Debug)]
pub struct History {
    history_iter: usize,
    history_file: fs::File,
    history_uncommitted: Option<String>,
    history: Vec<HistoryEntry>,
}

impl History {
    pub fn init(program_state: Rc<RefCell<state::ProgramState>>) -> Self {
        let p_state = program_state.borrow();
        let history_config = &p_state.config.history;
        let history_fp = p_state.config.core.data_dir.join(&history_config.history_fp);

        let history = Self::parse_history_file(&history_fp).unwrap();
        let history = Self::reduce_history_file(
            &history_fp, history_config.max_file_size_bytes, history).unwrap();
        let history_file = crate::utils::appendable_file(&history_fp).unwrap();

        Self {
            history_file,
            history,
            history_uncommitted: None,
            history_iter: 0,
        }
    }

    /// This function is expensive.
    /// This should be fine, since it will only run when the history file size has been exceeded.
    fn reduce_history_file(
        history_fp: &path::PathBuf,
        max_file_size_bytes: u64,
        history: Vec<HistoryEntry>,
    ) -> Result<Vec<HistoryEntry>, io::Error> {
        if !history_fp.exists() {
            panic!("Function called in bad order.")
        }

        if fs::metadata(&history_fp)?.len() < max_file_size_bytes {
            return Ok(history);
        }

        let mut entries_to_remove = vec![];
        for (i, history_entry) in history.iter().enumerate().rev().skip(1) {
            let mut remove = false;
            for already_entered in history[i + 1..history.len()].iter() {
                if history_entry.command == already_entered.command {
                    remove = true;
                    break;
                }
            }
            if remove {
                entries_to_remove.push(i);
            }
        }

        let temp_file_path = format!("{}.temp", history_fp.to_str().unwrap());
        let mut temp_file = fs::OpenOptions::new()
            .read(false)
            .append(true)
            .create(true)
            .open(&temp_file_path)?;

        let mut removal_counter = entries_to_remove.len() - 1;
        for (i, history_entry) in history.into_iter().enumerate() {
            if i == entries_to_remove[removal_counter] {
                if removal_counter > 0 {
                    removal_counter -= 1;
                }
                continue;
            }
            temp_file.write(history_entry.to_line().as_bytes())?;
        }
        drop(temp_file);

        fs::remove_file(&history_fp)?;
        fs::rename(&temp_file_path, &history_fp)?;

        Self::parse_history_file(&history_fp)
    }

    fn parse_history_file(history_fp: &path::PathBuf) -> Result<Vec<HistoryEntry>, io::Error> {
        if !history_fp.exists() {
            fs::File::create(history_fp)?;
        }

        let mut history = crate::utils::read_lines(history_fp)?
            .enumerate()
            .filter_map(|(line_number, line)| {
                if let Ok(line) = line {
                    if let Ok(history_entry) = HistoryEntry::from_line(&line, line_number as u32) {
                        return Some(history_entry);
                    }
                }
                None
            })
            .collect::<Vec<HistoryEntry>>();
        history.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(history)
    }

    pub fn add_to_history(&mut self, cmd: &str) -> anyhow::Result<()> {
        if cmd.len() == 0 {
            return Ok(());
        }

        let most_recent_history_line = if let Some(history_entry) = self.history.last() {
            history_entry.line_number
        } else {
            0
        };

        let current_timestamp = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs();

        let history_entry =
            HistoryEntry::new(current_timestamp, most_recent_history_line, cmd.to_string());

        self.history_file
            .write(history_entry.to_line().as_bytes())?;

        self.history.push(history_entry);

        self.history_uncommitted = None;
        self.history_iter = 0;

        Ok(())
    }

    /// None should be interpreted as: "keep whatever is currently in the `Buffer`".
    pub fn get_older_history(&mut self, cmd: &[char]) -> Option<&str> {
        if self.history.len() == 0 {
            return None;
        } else if self.history_iter == self.history.len() {
            return None;
        } else if self.history_iter == 0 {
            self.history_uncommitted = Some(cmd.iter().collect());
        }
        self.history_iter += 1;

        Some(&self.history[self.history.len() - self.history_iter].command)
    }

    pub fn get_newer_history(&mut self) -> Option<&str> {
        if self.history_iter == 0 {
            return None;
        } else if self.history_iter == 1 {
            self.history_iter -= 1;
            if let Some(partially_typed) = &self.history_uncommitted {
                return Some(partially_typed);
            }
            unreachable!("Partially typed command was not set?!?");
        }
        self.history_iter -= 1;
        Some(&self.history[self.history.len() - self.history_iter].command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history() {
        if path::PathBuf::from("history.testing").exists() {
            fs::remove_file("history.testing").unwrap();
        }
        fs::File::create("history.testing").unwrap();

        let history_config = history::ConfigHistory {
            history_fp: path::PathBuf::from("history.testing"),
            max_file_size_bytes: (16 + 1) * 3,
        };

        let history_config_2 = history::ConfigHistory {
            history_fp: path::PathBuf::from("history.testing"),
            max_file_size_bytes: (16 + 1) * 3,
        };

        match fs::remove_file(&history_config_2.history_fp) {
            Ok(_) => {}
            Err(_) => {}
        };

        let mut history = History::init(&history_config);
        assert_eq!(fs::metadata(&history_config_2.history_fp).unwrap().len(), 0);

        history.add_to_history("Hello").unwrap();
        let hello_time = history.history[0].timestamp;

        assert_eq!(
            fs::read_to_string(&history_config_2.history_fp).unwrap(),
            format!("{} Hello\n", hello_time)
        );

        history.add_to_history("World").unwrap();
        let world_time = history.history[1].timestamp;
        assert_eq!(
            fs::read_to_string(&history_config_2.history_fp).unwrap(),
            format!("{} Hello\n{} World\n", hello_time, world_time)
        );

        drop(history);

        assert_eq!(
            fs::metadata(&history_config_2.history_fp).unwrap().len(),
            (16 + 1) * 2
        );

        let mut history = History::init(&history_config);
        assert_eq!(history.history[0].timestamp, hello_time);
        assert_eq!(history.history[0].command, "Hello");
        assert_eq!(history.history[0].line_number, 0);

        assert_eq!(history.history[1].timestamp, world_time);
        assert_eq!(history.history[1].command, "World");
        assert_eq!(history.history[1].line_number, 1);

        history.add_to_history("World").unwrap();
        let new_world_time = history.history[2].timestamp;

        drop(history);
        assert_eq!(
            fs::metadata(&history_config_2.history_fp).unwrap().len(),
            (16 + 1) * 3
        );

        let mut history = History::init(&history_config);
        assert_eq!(
            fs::metadata(&history_config_2.history_fp).unwrap().len(),
            (16 + 1) * 2
        );
        fs::remove_file(&history_config_2.history_fp).unwrap();
    }

    #[test]
    fn test_history_recent_older() {
        if path::PathBuf::from("history.testing").exists() {
            fs::remove_file("history.testing").unwrap();
        }
        fs::File::create("history.testing").unwrap();
        let mut history_config = history::ConfigHistory {
            history_fp: path::PathBuf::from("history.testing"),
            max_file_size_bytes: (16 + 1) * 3,
        };

        let history_config_2 = history::ConfigHistory {
            history_fp: path::PathBuf::from("history.testing"),
            max_file_size_bytes: (16 + 1) * 3,
        };

        match fs::remove_file(&history_config_2.history_fp) {
            Ok(_) => {}
            Err(_) => {}
        };

        let mut history = History::init(&mut history_config);

        history.add_to_history("Hello").unwrap();
        history.add_to_history("World").unwrap();
        assert_eq!(
            history.get_older_history("CMD".chars().collect::<Vec<char>>().as_slice()),
            Some("World")
        );

        assert_eq!(history.history_iter, 1);
        assert_eq!(history.history_uncommitted, Some("CMD".to_string()));

        assert_eq!(
            history.get_older_history("World".chars().collect::<Vec<char>>().as_slice()),
            Some("Hello")
        );
        assert_eq!(history.history_iter, 2);
        assert_eq!(
            history.get_older_history("Hello".chars().collect::<Vec<char>>().as_slice()),
            None
        );
        assert_eq!(history.history_iter, 2);
        assert_eq!(history.get_newer_history(), Some("World"));
        assert_eq!(history.history_iter, 1);
        assert_eq!(history.get_newer_history(), Some("CMD"));
        assert_eq!(history.history_iter, 0);
        // assert_eq!(history.partially_typed, None);
        assert_eq!(history.get_newer_history(), None);
    }
}
