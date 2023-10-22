use std::cell::RefCell;
use std::path;
use std::process::exit;
use std::rc::Rc;
use crate::{buffer, enums, state};

use std::str::FromStr;

pub mod running;

pub type StatusCode = i32;

pub type ReservedFuncParams<'a> = (
    Rc<RefCell<state::ProgramState>>,
    &'a buffer::InputBuffer
);

pub enum ReservedFuncReturn {
    Ok,
    Status(StatusCode),
    DontExecute(StatusCode),
}

const RESERVED_COMMANDS: &[(&str, fn(ReservedFuncParams) -> ReservedFuncReturn)] = &[
    ("exit", exit_cmd),
    ("cd", cd_cmd),
    ("use", use_cmd)
];

fn get_nth(n: usize, buf: &buffer::InputBuffer) -> Option<String> {
    if n == 0 {
        panic!("This is 0-indexed");
    }
    if n > buf.num_args() {
        None
    } else {
        Some(buf.get_buffer_str(buf.arg_locs(n - 1)))
    }
}

fn remove_quotes(mut s: &str) -> &str {
    if s.starts_with('"') {
        s = &s[1..]
    }
    if s.ends_with('"') {
        s = &s[..s.len() - 1]
    }
    s
}

fn exit_cmd(params: ReservedFuncParams) -> ReservedFuncReturn {
    exit(0);
}

fn cd_cmd(params: ReservedFuncParams) -> ReservedFuncReturn {
    let (program_state, buf) = params;
    // TODO: Subsequent `cd`s?
    if let Some(dir) = get_nth(2, buf) {
        let clean = remove_quotes(&dir);
        let dir = path::PathBuf::from(clean);
        if dir.exists() {
            program_state.borrow_mut().current_working_directory = dir;
        } else {
            todo!("Display error")
        }
    }
    ReservedFuncReturn::DontExecute(0)
}

fn use_cmd(params: ReservedFuncParams) -> ReservedFuncReturn {
    let (program_state, buf) = params;
    if let Some(shell) = get_nth(2, buf) {
        let cleaned = remove_quotes(&shell);
        let shell = enums::Shell::from_str(cleaned);
        match shell {
            Ok(s) => program_state.borrow_mut().current_shell = s,
            Err(_) => todo!()
        }
    }
    ReservedFuncReturn::DontExecute(0)
}
