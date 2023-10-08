/// This module is a "backend" to multiple processes.
/// IPC is done via a TCP connection.
/// Multiple processes of whale_rs can connect to this TCP server.
/// The TCP server can be run as a daemon // TODO
/// Or the TCP server is started by the first process that finds there isn't a TCP server running.
/// If the owning process dies, the TCP server will die with it, and another instance can start one.

use std::{fs, io::{self, Write}, path, thread, time};
use std::cell::RefCell;
use std::io::Read;
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use crate::history::BUFFER_SIZE;
use crate::state;
use super::HistoryEntry;


#[derive(Serialize, Deserialize)]
pub enum HistoryRequest {
    Exit,
    AddToHistory(String),
    FindFirstOccurrence(String),
    GetHistoryInd(usize),
    GetNumHistoryEntries,
}

#[derive(Serialize, Deserialize)]
pub enum HistoryResponse {
    HistoryVal(Option<HistoryEntry>),
    HistoryInd(usize),
    Ok,
}

pub fn start_history_data_layer(program_state: Rc<RefCell<state::ProgramState>>) -> anyhow::Result<()> {
    let (listener, history) = {
        let p_state = program_state.borrow();
        let port = p_state.config.history.tcp_port;
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
        let history = Arc::new(RwLock::new(DataLayerHistory::init(program_state.clone())));
        (listener, history)
    };

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let history = history.clone();
                    thread::spawn(move || {
                        handle_request(stream, history)
                    });
                }
                Err(_e) => (),
            }
        }
    });

    Ok(())
}

fn handle_request(stream: std::net::TcpStream, history: Arc<RwLock<DataLayerHistory>>) {
    let mut stream = stream;
    // Buffer is 2**13 for typing commands, so this should be fine for now.
    let mut buffer = [0; BUFFER_SIZE];
    loop {
        let bytes = stream.read(&mut buffer).unwrap();
        if bytes == 0 {
            break;
        }
        let request: HistoryRequest = bincode::deserialize(&buffer[..bytes]).unwrap();
        let mut exit = false;
        let resp = match request {
            HistoryRequest::Exit => {
                exit = true;
                HistoryResponse::Ok
            }
            HistoryRequest::AddToHistory(command) => {
                let mut history = history.write().unwrap();
                history.add_to_history(&command).unwrap();
                HistoryResponse::Ok
            }
            HistoryRequest::FindFirstOccurrence(command) => {
                let history = history.read().unwrap();
                let history_entry = history.find_first(&command);
                HistoryResponse::HistoryVal(history_entry.cloned())
            }
            HistoryRequest::GetNumHistoryEntries => {
                let history = history.read().unwrap();
                HistoryResponse::HistoryInd(history.len())
            }
            HistoryRequest::GetHistoryInd(ind) => {
                let mut history = history.write().unwrap();
                HistoryResponse::HistoryVal(history.get_history(ind).cloned())
            }
        };
        let resp = bincode::serialize(&resp).unwrap();
        stream.write(&resp).unwrap();

        if exit {
            break;
        }
    }
}

#[derive(Debug)]
struct DataLayerHistory {
    history_file: fs::File,
    history: Vec<HistoryEntry>,
}

impl DataLayerHistory {
    fn init(program_state: Rc<RefCell<state::ProgramState>>) -> Self {
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
        }
    }

    fn len(&self) -> usize {
        self.history.len()
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

        let current_timestamp = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs();

        let history_entry = HistoryEntry::new(current_timestamp, cmd.to_string());

        self.history_file.write(history_entry.to_line().as_bytes())?;
        self.history.push(history_entry);

        Ok(())
    }

    /// None should be interpreted as: "keep whatever is currently in the `Buffer`".
    pub fn get_history(&mut self, ind: usize) -> Option<&HistoryEntry> {
        if self.history.len() == 0 {
            return None;
        } else if ind >= self.history.len() {
            return None;
        }
        Some(&self.history[ind])
    }

    pub fn find_first(&self, cmd: &str) -> Option<&HistoryEntry> {
        for history_entry in self.history.iter().rev() {
            if history_entry.command.starts_with(cmd) {
                return Some(history_entry);
            }
        }
        None
    }
}
