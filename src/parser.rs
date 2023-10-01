use std::cell::{Ref, RefCell};
use std::path;
use std::rc::Rc;
use crate::{buffer, enums, state};
use crate::config::command;
use crate::hints::Disregard;

// #[derive(Debug)]
// enum Skip<'a> {
//     None,
//     Once(Argument<'a>),
//     Twice(Argument<'a>),
// }

#[derive(Debug)]
pub enum Argument<'a> {
    Other,
    // Flag(&'a command::Flag),
    // Arg(&'a command::SingleArg),
    // ArgFlag(&'a command::FlagArgPair),
    Flag(Ref<'a, command::Flag>),
    Arg(Ref<'a, command::SingleArg>),
    ArgFlag(Ref<'a, command::FlagArgPair>),
}

pub struct BufferParser<'a> {
    arg_ind: usize,
    single_argument_count: usize,
    current_cmd: Option<Ref<'a, command::ConfigCommand>>,
    current_cmd_ind: Option<usize>,
    program_state: Rc<RefCell<state::ProgramState>>,
    args: Vec<String>,

    flag_skips: Vec<usize>,
    arg_skips: Vec<usize>,
    arg_flag_skips: Vec<usize>,
}

impl<'a> BufferParser<'a> {
    pub fn new(
        program_state: Rc<RefCell<state::ProgramState>>,
    ) -> Self {
        Self {
            arg_ind: 0,
            single_argument_count: 0,
            current_cmd: None,
            current_cmd_ind: None,
            program_state,
            args: vec![],
            flag_skips: vec![],
            arg_skips: vec![],
            arg_flag_skips: vec![],
        }
    }

    pub fn init(&'a mut self, buf: &buffer::InputBuffer) {
        self.arg_ind = 0;
        self.single_argument_count = 0;
        self.flag_skips.clear();
        self.arg_skips.clear();
        self.arg_flag_skips.clear();
        self.args = buf
            .arg_locs_iterator()
            .map(|range| buf.get_buffer_str(range))
            .collect::<Vec<_>>();

        let first_arg = {
            if buf.num_args() == 0 {
                self.current_cmd = None;
                self.current_cmd_ind = None;
                return;
            }
            buf.get_buffer_str(buf.arg_locs(0))
        };

        if !first_arg.is_empty() {
            if self.current_cmd.is_some() {
                if first_arg != self.current_cmd.as_ref().unwrap().exe_name {
                    self.current_cmd = None;
                    self.current_cmd_ind = None;
                }
            }
            if self.current_cmd.is_none() {
                let mut index = None;
                for (i, cmd) in self.program_state.borrow().config.commands.iter().enumerate() {
                    if cmd.exe_name == first_arg {
                        index = Some(i);
                    }
                }
                if let Some(i) = index {
                    self.current_cmd = Some(
                        Ref::map(self.program_state.borrow(),
                                 |s| &s.config.commands[i]));
                    self.current_cmd_ind = Some(i);
                }
            }
        }

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
            true => self.program_state.borrow().current_working_directory.join(fp),
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
    ) -> Option<Argument> {
        for (k, arg_flag) in self.current_cmd.as_ref().unwrap().arg_flags.iter().enumerate() {
            if self.arg_flag_skips.contains(&k) {
                continue;
            }
            if arg_flag.flag_name == arg {
                // arg_ind += 1 because we want to skip the next arg (pair of flag and arg)
                self.arg_ind += 1;
                self.arg_flag_skips.push(k);
                return Some(Argument::ArgFlag(Ref::map(self.program_state.borrow(),
                                                       |s| &s.config.commands[self.current_cmd_ind.unwrap()].arg_flags[k])));
                // return Some(Argument::ArgFlag(arg_flag));
            }
        }
        None
    }

    /// Skip behaviour = Once
    fn process_flags(
        &mut self,
        arg: &str,
    ) -> Option<Argument> {
        for (k, flag) in self.current_cmd.as_ref().unwrap().flags.iter().enumerate() {
            if self.flag_skips.contains(&k) {
                continue;
            }
            if flag.flag_name == arg {
                self.flag_skips.push(k);
                return Some(Argument::Flag(Ref::map(self.program_state.borrow(),
                                                    |s| &s.config.commands[self.current_cmd_ind.unwrap()].flags[k])));
                // return Some(Argument::Flag(flag));
            }
        }
        None
    }

    /// Skip behaviour = Once
    fn process_args(
        &mut self,
    ) -> Option<Argument> {
        for (k, single_arg) in self.current_cmd.as_ref().unwrap().args.iter().enumerate() {
            if self.arg_skips.contains(&k) {
                continue;
            }
            if single_arg.arg_pos == self.single_argument_count {
                self.single_argument_count += 1;
                self.arg_skips.push(k);
                return Some(Argument::Arg(Ref::map(self.program_state.borrow(),
                                                   |s| &s.config.commands[self.current_cmd_ind.unwrap()].args[k])));
                // return Some(Argument::Arg(single_arg));
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
impl<'a> Iterator for BufferParser<'a> {
    type Item = Argument<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.is_empty() {
            return None;
        }

        if self.arg_ind == self.args.len() {
            return None;
        }

        if self.current_cmd.is_none() {
            return Some(Argument::Other);
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

        let skip: Argument;

        if let Some(arg_flag) = self.process_arg_flags(&arg) {
            skip = arg_flag;
        } else if let Some(flag) = self.process_flags(&arg) {
            skip = flag;
        } else if let Some(single_arg) = self.process_args() {
            skip = single_arg;
        } else {
            skip = Argument::Other;
        }

        Some(skip)
    }
}
