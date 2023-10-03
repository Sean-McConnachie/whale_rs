use std::cell::{Ref, RefCell};
use std::rc::Rc;
use crate::{state, utils};
use crate::config::command;


// I spent two days trying to get this working with references, only to say fuck it and go to pointers
// Code an certainly be improved, I'm going to leave it for now.
// TODO: Test the performance impact of clones for scenarios like this.
#[derive(Debug)]
pub enum Argument {
    Other,
    Flag(*const command::Flag),
    Arg(*const command::SingleArg),
    ArgFlag(*const command::FlagArgPair),
}

#[derive(Debug)]
pub struct ArgumentParser {
    has_command: bool,
    program_state: Rc<RefCell<state::ProgramState>>,
    current_cmd: command::ConfigCommand,
    first_arg: String,
}

impl ArgumentParser {
    pub fn new(program_state: Rc<RefCell<state::ProgramState>>) -> Self {
        Self {
            has_command: false,
            program_state,
            current_cmd: command::ConfigCommand::default(),
            first_arg: String::new(),
        }
    }

    pub fn reinit(&mut self, first_arg: Option<String>) {
        if let Some(arg) = first_arg {
            if self.first_arg != arg {
                self.first_arg = arg.clone();
            }
        } else {
            self.has_command = false;
            return;
        }

        if !self.first_arg.is_empty() {
            if self.has_command {
                if self.first_arg != self.current_cmd.exe_name {
                    self.has_command = false;
                }
            }
            if !self.has_command {
                for cmd in self.program_state.borrow().config.commands.iter() {
                    if cmd.exe_name == self.first_arg {
                        self.has_command = true;
                        // This clone only gets called when a new command is typed (i.e. the exe)
                        self.current_cmd = cmd.clone();
                        break;
                    }
                }
            }
        }
    }

    pub fn cmd(&self) -> &command::ConfigCommand {
        &self.current_cmd
    }

    pub fn has_command(&self) -> bool {
        self.has_command
    }

    pub fn first_arg(&self) -> &str {
        &self.first_arg
    }
}

#[derive(Debug)]
pub struct ArgumentIterator<'a> {
    argument_parser: &'a ArgumentParser,
    arg_ind: usize,
    single_argument_count: usize,
    args: Vec<String>,
    flag_skips: Vec<usize>,
    arg_flag_skips: Vec<usize>,
}

impl<'a> ArgumentIterator<'a> {
    pub fn new(argument_parser: &'a ArgumentParser) -> Self {
        Self {
            argument_parser,
            arg_ind: 0,
            single_argument_count: 0,
            args: vec![],
            flag_skips: vec![],
            arg_flag_skips: vec![],
        }
    }

    pub fn reinit(&mut self, args: Vec<String>) {
        self.arg_ind = 0;
        self.single_argument_count = 0;
        self.flag_skips.clear();
        self.arg_flag_skips.clear();
        self.args = args;
    }

    /// Skip behaviour = Twice
    fn process_arg_flags(
        &mut self,
        arg: &str,
    ) -> Option<Argument> {
        let k = utils::binary_search_with_exclude(
            arg,
            command::FlagArgPair::flag_name,
            &self.argument_parser.cmd().arg_flags,
            &self.arg_flag_skips,
        );

        if let Some(k) = k {
            // arg_ind += 1 because we want to skip the next arg (pair of flag and arg)
            self.arg_ind += 1;
            self.arg_flag_skips.push(k);
            return Some(Argument::ArgFlag(&self.argument_parser.cmd().arg_flags[k] as *const command::FlagArgPair));
        }
        None
    }

    /// Skip behaviour = Once
    fn process_flags(
        &mut self,
        arg: &str,
    ) -> Option<Argument> {
        let k = utils::binary_search_with_exclude(
            arg,
            command::Flag::flag_name,
            &self.argument_parser.cmd().flags,
            &self.flag_skips,
        );

        if let Some(k) = k {
            self.flag_skips.push(k);
            return Some(Argument::Flag(&self.argument_parser.cmd().flags[k] as *const command::Flag));
        }
        None
    }

    /// Skip behaviour = Once
    fn process_args(
        &mut self,
    ) -> Option<Argument> {
        if self.single_argument_count == self.argument_parser.cmd().args.len() {
            return None;
        }

        let arg = &self.argument_parser.cmd().args[self.single_argument_count];
        self.single_argument_count += 1;
        return Some(Argument::Arg(arg as *const command::SingleArg));
    }
}

/// Note that `arg_ind` plays a large role in this iterator. It is used to keep track of the current
/// argument that is being processed and is responsible for skipping arguments.
impl<'a> Iterator for ArgumentIterator<'a> {
    type Item = Argument;
    fn next(&mut self) -> Option<Self::Item> {
        if self.args.is_empty() {
            return None;
        }

        if self.arg_ind == self.args.len() {
            return None;
        }

        if self.arg_ind == 0 {
            self.arg_ind += 1;
            self.single_argument_count += 1;
            // `command.rs` guarantees that there will always be at least one arg, the executable
            return Some(Argument::Arg(&self.argument_parser.cmd().args[0] as *const command::SingleArg));
        }

        let arg = self.args[self.arg_ind].clone();

        self.arg_ind += 1;

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
