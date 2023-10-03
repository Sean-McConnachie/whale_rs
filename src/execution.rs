use std::cell::RefCell;
use std::rc::Rc;
use crate::{ansi, buffer, parser, state};

// TODO: Add support for execute_before and execute_after
pub fn run_command(
    program_state: Rc<RefCell<state::ProgramState>>,
    buffer: &buffer::InputBuffer,
    arg_parser: &parser::ArgumentParser,
) -> (Option<u16>, Option<i32>) {
    if buffer.len() == 0 { return (None, Some(-1)); }

    // TODO: Add a mutable `current_shell`.
    let mut command = program_state.borrow().config.core.default_shell.to_exec();

    let args = buffer.arg_locs_iterator()
        .map(|range| buffer.get_buffer_str(range))
        .collect::<Vec<_>>();

    let mut shell_str = String::new();
    if !arg_parser.has_command() {
        for split in buffer.arg_locs_iterator() {
            shell_str += &buffer.get_buffer_str(split);
            shell_str += " ";
        }
    } else {
        let mut iter = parser::ArgumentIterator::new(&arg_parser);
        iter.reinit(args.clone());
        let mut i = 0;
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
                parser::Argument::Arg(_arg) => { shell_str += &args[i]; }
            }
            shell_str += " ";
            i += 1;
        }
    }
    command.arg(shell_str);

    println!();
    ansi::move_to_column(0);
    ansi::erase_screen_from_cursor();
    ansi::flush();

    crossterm::terminal::disable_raw_mode().unwrap();

    let mut child = command.spawn().unwrap();
    let exit = child.wait().unwrap();

    crossterm::terminal::enable_raw_mode().unwrap();

    // TODO: Terminal position updates
    let new_pos = ansi::cursor_pos().unwrap();

    return (Some(new_pos.1), exit.code());
}