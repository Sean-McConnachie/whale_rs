// use std::process;
// use crate::{buffer, enums::Shell, state};
//
//
// pub fn run_command(program_state: &state::ProgramState, buffer: &buffer::InputBuffer) -> Option<i32> {
//     if buffer.len() == 0 { return Some(-1); }
//
//     // TODO: Add a mutable `current_shell`.
//     let mut command = program_state.config.core.default_shell.to_exec();
//
//     if let Some(cmd_conf) = buffer.get_current_command() {
//         //
//     } else {
//         for split in buffer.arg_locs_iterator() {
//             command.arg(buffer.get_buffer_str(split));
//         }
//     }
//
//     for arg_c in 1..command_buffer.arg_c() {
//         if let Some(arg) = command_buffer.arg(arg_c) {
//             let arg = CommandBuffer::chars_to_str(arg);
//             command.arg(arg.trim());
//         }
//     }
//
//     println!();
//     crossterm::terminal::disable_raw_mode().unwrap();
//
//     let mut child = command.spawn().unwrap();
//     let exit = child.wait().unwrap();
//
//     crossterm::terminal::enable_raw_mode().unwrap();
//
//     let new_pos = Style::get_cursor_pos();
//     ConsoleStyle::set_line_number(new_pos.0);
//
//     return exit.code();
// }