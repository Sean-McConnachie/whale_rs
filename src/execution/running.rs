use std::cell::RefCell;
use std::rc::Rc;
use crate::{ansi, buffer, enums, parser, state};

fn reserved_commands(
    program_state: Rc<RefCell<state::ProgramState>>,
    buffer: &buffer::InputBuffer,
) -> Option<super::ReservedFuncReturn> {
    let first_arg = buffer.first_arg().unwrap_or_default();
    for (cmd, func) in super::RESERVED_COMMANDS {
        if &first_arg == cmd {
            return Some(func((program_state.clone(), buffer)));
        }
    }
    None
}

fn parse_execution_cmd(args: &Vec<String>, command: &str) -> String {
    let to_usize = |s: &str| -> usize {
        s.parse::<usize>().unwrap()
    };
    let mut cmd = String::with_capacity(command.len());
    for cmd_arg in command.split(' ') {
        if cmd_arg.starts_with('$') {
            let argv = &cmd_arg[1..];
            match (argv.starts_with(".."), argv.ends_with(".."), argv.len()) {
                (true, true, _) => cmd += &args.join(" "),
                (true, false, l) => {
                    let argv = to_usize(&argv[2..]).min(l);
                    cmd += &args[..argv].join(" ");
                }
                (false, true, l) => {
                    let argv = to_usize(&argv[..l - 2]).min(l);
                    cmd += &args[argv..].join(" ");
                }
                (false, false, l) => {
                    if argv.contains("..") {
                        let argv = argv.split("..").collect::<Vec<_>>();
                        let start = to_usize(argv[0]).min(l);
                        let end = to_usize(argv[1]).min(l);
                        cmd += &args[start..end].join(" ");
                    } else {
                        let argv = to_usize(argv);
                        cmd += &args[argv..argv + 1].join(" ");
                    }
                }
            }
        } else {
            cmd += cmd_arg;
        }
        cmd += " ";
    }
    cmd.pop();
    cmd
}

pub type NewTerminalLine = u16;

pub fn run_command(
    program_state: Rc<RefCell<state::ProgramState>>,
    buffer: &buffer::InputBuffer,
    arg_parser: &parser::ArgumentParser,
) -> (Option<NewTerminalLine>, Option<super::StatusCode>) {
    match reserved_commands(
        program_state.clone(),
        buffer,
    ) {
        None => (),
        Some(action) => match action {
            super::ReservedFuncReturn::Ok => (),
            super::ReservedFuncReturn::Status(_) => (),
            super::ReservedFuncReturn::DontExecute(_) => return (None, None)
        }
    };

    if buffer.len() == 0 { return (None, Some(-1)); }

    let p_state = program_state.borrow();

    let command_strs = { // Construct the commands
        let args = buffer.arg_locs_iterator()
            .map(|range| buffer.get_buffer_str(range))
            .collect::<Vec<_>>();

        let mut command_strs = vec![];
        if !arg_parser.has_command() {
            let mut shell_str = String::with_capacity(buffer.len());
            for split in buffer.arg_locs_iterator() {
                shell_str += &buffer.get_buffer_str(split);
                shell_str += " ";
            }
            shell_str.pop();
            command_strs.push(shell_str);
        } else {
            if let Some(cmd) = &arg_parser.cmd().execute_before {
                command_strs.push(parse_execution_cmd(&args, cmd));
            }

            let mut iter = parser::ArgumentIterator::new(&arg_parser);
            iter.reinit(args.clone());
            let mut i = 0;
            let mut shell_str = String::with_capacity(buffer.len());
            shell_str += &arg_parser.cmd().exe_to;
            shell_str += " ";
            let _ = iter.next();
            for arg in iter {
                match arg {
                    parser::Argument::Other => { shell_str += &args[i]; }
                    parser::Argument::ArgFlag(arg_flag) => {
                        let arg_flag = unsafe { &*arg_flag };
                        shell_str += &arg_flag.flag_to;
                        i += 1;
                        shell_str += " ";
                        shell_str += &args[i];
                    }
                    parser::Argument::Flag(flag) => {
                        let flag = unsafe { &*flag };
                        shell_str += &flag.flag_to;
                    }
                    parser::Argument::Arg(arg) => { shell_str += &args[i]; }
                }
                shell_str += " ";
                i += 1;
            }
            shell_str.pop();
            command_strs.push(shell_str);

            if let Some(cmd) = &arg_parser.cmd().execute_after {
                command_strs.push(parse_execution_cmd(&args, cmd));
            }
        }
        command_strs
    };

    { // Run the commands
        let run_cmd = |exec_str: &str| -> std::process::ExitStatus {
            let mut command = p_state.current_shell.to_exec();
            command.arg(exec_str);
            let mut child = command.spawn().unwrap();
            let exit = child.wait().unwrap();
            exit
        };

        println!();
        ansi::move_to_column(0);
        ansi::erase_screen_from_cursor();
        ansi::flush();

        crossterm::terminal::disable_raw_mode().unwrap();

        let mut exit = run_cmd(&command_strs[0]);
        for cmd in &command_strs[1..] {
            exit = run_cmd(cmd);
        }

        crossterm::terminal::enable_raw_mode().unwrap();

        let new_pos = ansi::cursor_pos().unwrap();

        return (Some(new_pos.1), exit.code());
    }
}