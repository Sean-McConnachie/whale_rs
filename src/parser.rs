use std::cell::{Ref, RefCell};
use std::path;
use std::rc::Rc;
use crate::{buffer, state};
use crate::config::command;
use crate::hints::Disregard;

#[derive(Debug)]
pub enum Argument<'a> {
    Other,
    Flag(&'a command::Flag),
    Arg(&'a command::SingleArg),
    ArgFlag(&'a command::FlagArgPair),
}

pub struct BufferParser<'a> {
    program_state: Rc<RefCell<state::ProgramState>>,
    current_cmd: command::ConfigCommand,
    parser: Option<Parser<'a>>,
}

impl<'a> BufferParser<'a> {
    pub fn new(program_state: Rc<RefCell<state::ProgramState>>) -> Self {
        Self {
            program_state,
            current_cmd: command::ConfigCommand::default(),
            parser: None,
        }
    }

    pub fn init(&'a mut self, buf: &buffer::InputBuffer) {
        let first_arg = {
            if buf.num_args() == 0 {
                self.current_cmd = command::ConfigCommand::default();
                self.parser = None;
                return;
            }
            buf.get_buffer_str(buf.arg_locs(0))
        };

        if !first_arg.is_empty() {
            if self.parser.is_some() {
                if first_arg != self.current_cmd.exe_name {
                    self.current_cmd = command::ConfigCommand::default();
                    self.parser = None;
                }
            }
            if self.parser.is_none() {
                for cmd in self.program_state.borrow().config.commands.iter() {
                    if cmd.exe_name == first_arg {
                        self.current_cmd = cmd.clone();
                        let cwd = self.program_state.borrow().current_working_directory.clone();
                        self.parser = Some(Parser::new(&self.current_cmd, cwd));
                        break;
                    }
                }
            }
            if self.parser.is_some() {
                let cwd = self.program_state.borrow().current_working_directory.clone();
                self.parser.as_mut().unwrap().init(buf, cwd);
            }
        }
    }
}

struct Parser<'a> {
    current_cmd: &'a command::ConfigCommand,
    cwd: path::PathBuf,
    arg_ind: usize,
    single_argument_count: usize,
    args: Vec<String>,
    flag_skips: Vec<usize>,
    arg_skips: Vec<usize>,
    arg_flag_skips: Vec<usize>,
}

impl<'a> Parser<'a> {
    pub fn new(current_cmd: &'a command::ConfigCommand, cwd: path::PathBuf) -> Self {
        Self {
            current_cmd,
            cwd,
            arg_ind: 0,
            single_argument_count: 0,
            args: vec![],
            flag_skips: vec![],
            arg_skips: vec![],
            arg_flag_skips: vec![],
        }
    }

    pub fn init(&mut self, buf: &buffer::InputBuffer, cwd: path::PathBuf) {
        self.cwd = cwd;
        self.arg_ind = 0;
        self.single_argument_count = 0;
        self.flag_skips.clear();
        self.arg_skips.clear();
        self.arg_flag_skips.clear();
        self.args = buf
            .arg_locs_iterator()
            .map(|range| buf.get_buffer_str(range))
            .collect::<Vec<_>>();

        if !self.args.is_empty() { // Skip the first arg if it's empty
            self.args.remove(0);
        }
    }

    // TODO: Fix the `Hint`ing system... These return types are just stupid
    fn arg_to_path(
        &self,
        s: &str,
    ) -> Option<(path::PathBuf, Disregard, String)> {
        let fp = path::PathBuf::from(s);

        let last = if !s.is_empty() {
            fp.iter().last().unwrap().len()
        } else {
            0
        };
        let disregard = s.len() - last;

        let fp = match fp.is_relative() {
            true => self.cwd.join(fp),
            false => fp,
        };

        let mut cleaned_path = path::PathBuf::new();
        for dir in fp.iter() {
            if dir == ".." {
                let _ = cleaned_path.pop();
            } else {
                cleaned_path.push(dir);
            }
        }

        if cleaned_path.is_dir() {
            return Some((cleaned_path, disregard, s[disregard..].to_string()));
        }
        if let Some(p) = cleaned_path.parent() {
            if p.is_dir() {
                return Some((cleaned_path.parent().unwrap().to_path_buf(), disregard, s[disregard..].to_string()));
            }
        }
        None
    }

    /// Skip behaviour = Twice
    fn process_arg_flags(
        &mut self,
        arg: &str,
    ) -> Option<Argument<'a>> {
        for (k, arg_flag) in self.current_cmd.arg_flags.iter().enumerate() {
            if self.arg_flag_skips.contains(&k) {
                continue;
            }
            if arg_flag.flag_name == arg {
                // arg_ind += 1 because we want to skip the next arg (pair of flag and arg)
                self.arg_ind += 1;
                self.arg_flag_skips.push(k);
                return Some(Argument::ArgFlag(arg_flag));
            }
        }
        None
    }

    /// Skip behaviour = Once
    fn process_flags(
        &mut self,
        arg: &str,
    ) -> Option<Argument<'a>> {
        for (k, flag) in self.current_cmd.flags.iter().enumerate() {
            if self.flag_skips.contains(&k) {
                continue;
            }
            if flag.flag_name == arg {
                self.flag_skips.push(k);
                return Some(Argument::Flag(flag));
            }
        }
        None
    }

    /// Skip behaviour = Once
    fn process_args(
        &mut self,
    ) -> Option<Argument<'a>> {
        for (k, single_arg) in self.current_cmd.args.iter().enumerate() {
            if self.arg_skips.contains(&k) {
                continue;
            }
            if single_arg.arg_pos == self.single_argument_count {
                self.single_argument_count += 1;
                self.arg_skips.push(k);
                return Some(Argument::Arg(single_arg));
            }
        }
        None
    }

    fn safe_pop_front(&mut self) {
        if self.args.is_empty() {
            return;
        }
        self.args.remove(0);
    }
}

/// Note that `arg_ind` plays a large role in this iterator. It is used to keep track of the current
/// argument that is being processed and is responsible for skipping arguments.
impl<'a> Iterator for Parser<'a> {
    type Item = Argument<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.is_empty() {
            return None;
        }

        if self.arg_ind == self.args.len() {
            return None;
        }

        if self.arg_ind == 0 {
            self.arg_ind += 1;
            // `command.rs` guarantees that there will always be at least one arg, the executable
            // return Some(Argument::Arg(&self.current_cmd.as_ref().unwrap().args[0])); todo
            return Some(Argument::Other);
        }

        let arg = self.args[self.arg_ind].clone();

        self.arg_ind += 1;

        // TODO: Use binary searches instead

        let rtn = self.process_arg_flags(&arg);
        if rtn.is_some() {
            return rtn;
        }

        let rtn = self.process_flags(&arg);
        if rtn.is_some() {
            return rtn;
        }

        let rtn = self.process_args();
        if rtn.is_some() {
            return rtn;
        }

        return Some(Argument::Other);
    }
}
