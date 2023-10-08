use std::time;
use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::rc::Rc;
use crate::history::{BUFFER_SIZE, HistoryEntry};
use crate::history::data_layer::{HistoryRequest, HistoryResponse};
use crate::state;

#[derive(Debug)]
pub struct History {
    program_state: Rc<RefCell<state::ProgramState>>,
    data_conn: TcpStream,
    buffer: [u8; BUFFER_SIZE],
    history_iter: usize,
    history_uncommitted: Option<HistoryEntry>,
}

impl History {
    pub fn test_and_fix_connection(&mut self) {
        match self.data_conn.peer_addr() {
            Ok(_) => {
                return;
            }
            Err(_e) => {
                super::data_layer::start_history_data_layer(self.program_state.clone())
                    .unwrap();
                let port = self.program_state.borrow().config.history.tcp_port;
                self.data_conn = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
            }
        }
    }

    pub fn init(program_state: Rc<RefCell<state::ProgramState>>) -> Self {
        let data_conn = {
            let p_state = program_state.borrow();
            let port = p_state.config.history.tcp_port;
            let addr = format!("127.0.0.1:{}", port);
            match TcpStream::connect(&addr) {
                Ok(stream) => stream,
                Err(_no_server) => {
                    super::data_layer::start_history_data_layer(program_state.clone())
                        .unwrap();
                    TcpStream::connect(addr).unwrap()
                }
            }
        };
        let mut s = Self {
            program_state: program_state.clone(),
            data_conn,
            buffer: [0; BUFFER_SIZE],
            history_uncommitted: None,
            history_iter: 0,
        };
        s.update_oldest_ind();
        s
    }

    fn write_buf(&mut self, data: HistoryRequest) -> Result<usize, std::io::Error> {
        self.data_conn.write(&bincode::serialize(&data).unwrap())
    }

    fn read_buf(&mut self) -> Result<HistoryResponse, std::io::Error> {
        let bytes = self.data_conn.read(&mut self.buffer)?;
        Ok(bincode::deserialize(&self.buffer[..bytes]).unwrap())
    }

    fn update_oldest_ind(&mut self) {
        self.test_and_fix_connection();

        self.write_buf(HistoryRequest::GetNumHistoryEntries).unwrap();
        match self.read_buf().unwrap() {
            HistoryResponse::HistoryInd(ind) => {
                self.history_iter = ind;
            }
            _ => unreachable!("Invalid response from history data layer"),
        }
    }

    pub fn add_to_history(&mut self, cmd: String) -> anyhow::Result<()> {
        if cmd.len() == 0 {
            return Ok(());
        }

        self.test_and_fix_connection();

        self.write_buf(HistoryRequest::AddToHistory(cmd)).unwrap();
        match self.read_buf().unwrap() {
            HistoryResponse::Ok => (),
            _ => unreachable!("Invalid response from history data layer"),
        }

        self.history_uncommitted = None;
        self.history_iter = 0;

        Ok(())
    }

    pub fn get_older_history(&mut self, cmd: &[char]) -> Option<HistoryEntry> {
        self.test_and_fix_connection();

        if self.history_iter > 0 {
            if self.history_uncommitted.is_none() {
                self.history_uncommitted = Some(HistoryEntry::new(0, cmd.iter().collect()));
            }
            self.history_iter -= 1;

            self.write_buf(HistoryRequest::GetHistoryInd(self.history_iter)).unwrap();
            match self.read_buf().unwrap() {
                HistoryResponse::HistoryVal(history_entry) => {
                    return history_entry;
                }
                _ => unreachable!("Invalid response from history data layer"),
            }
        }
        None
    }

    pub fn get_newer_history(&mut self) -> Option<HistoryEntry> {
        self.test_and_fix_connection();

        self.write_buf(HistoryRequest::GetHistoryInd(self.history_iter)).unwrap();
        match self.read_buf().unwrap() {
            HistoryResponse::HistoryVal(history_entry) => {
                if history_entry.is_some() {
                    return history_entry;
                }
            }
            _ => unreachable!("Invalid response from history data layer"),
        }
        {
            // we have probably gone out of range. But this could be because the file has been
            // concatenated by a new instance of the server. So instead we query for the highest index.
            self.update_oldest_ind();
            let r = self.history_uncommitted.clone();
            self.history_uncommitted = None;
            return r; // purposeful unwrap
        }
    }
}
