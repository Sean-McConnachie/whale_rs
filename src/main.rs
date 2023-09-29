use whale_rs::input::{InputEvent};
use whale_rs::buffer::Side;
use whale_rs::{ansi, buffer, config, gui, input, state};

enum AdditionalViewAction {
    None,
    SetTo(gui::AdditionalViewNoData),
    Unset,
}

fn update_buffer<'a>(
    input: InputEvent,
    buffer: &'a mut buffer::InputBuffer,
    term_size: &mut ansi::TerminalXY,
    terminal_gui: &gui::terminal::TerminalGUI,
) -> AdditionalViewAction {
    let mut rtn = AdditionalViewAction::None;
    match input {
        InputEvent::Esc => buffer.unset_secondary_cursor(),
        InputEvent::Backspace => buffer.del_n(Side::Left, 1),
        InputEvent::Delete => buffer.del_n(Side::Right, 1),
        InputEvent::Enter => {
            // TODO: Run
            // buffer.enter();
            // command_run = true;
        }
        InputEvent::Tab => {
            let hints = buffer.get_argument_hints();
            let curr = buffer.arg_locs(buffer.get_curr_arg());
            let hint = &hints[buffer.get_curr_arg()].1;
            let hint_arg = hint.last_closest_match().unwrap()[(curr.1 - curr.0 - hint.disregard())..].to_string();
            buffer.insert_str_main_cursor(&hint_arg);
        }
        InputEvent::Character(c) => {
            buffer.del_betw_curs();
            buffer.insert_char_main_cursor(c);
        }
        InputEvent::CtrlBackspace => buffer.del_jump(Side::Left),
        InputEvent::CtrlDelete => buffer.del_jump(Side::Left),
        InputEvent::CtrlC => unreachable!("This should be handled outside of the match statement!"),

        InputEvent::ArrowLeft => {
            buffer.main_cur_set(buffer.move_n(Side::Left, 1, buffer.main_cur()));
            buffer.unset_secondary_cursor();
        }
        InputEvent::ArrowRight => {
            buffer.main_cur_set(buffer.move_n(Side::Right, 1, buffer.main_cur()));
            buffer.unset_secondary_cursor();
        }

        InputEvent::CtrlArrowLeft => {
            buffer.main_cur_set(buffer.jump(Side::Left, buffer.main_cur()));
            buffer.unset_secondary_cursor();
        }
        InputEvent::CtrlArrowRight => {
            buffer.main_cur_set(buffer.jump(Side::Right, buffer.main_cur()));
            buffer.unset_secondary_cursor();
        }

        InputEvent::AltArrowLeft => {
            buffer.enable_sec_cur_if_not_active();
            let new_pos = buffer.move_n(Side::Left, 1, buffer.sec_cur());
            buffer.sec_cur_set(new_pos, true)
        }
        InputEvent::AltArrowRight => {
            buffer.enable_sec_cur_if_not_active();
            let new_pos = buffer.move_n(Side::Right, 1, buffer.sec_cur());
            buffer.sec_cur_set(new_pos, true);
        }

        InputEvent::CtrlShiftArrowLeft => {
            buffer.enable_sec_cur_if_not_active();
            let new_pos = buffer.jump(Side::Left, buffer.sec_cur());
            buffer.sec_cur_set(new_pos, true)
        }
        InputEvent::CtrlShiftArrowRight => {
            buffer.enable_sec_cur_if_not_active();
            let new_pos = buffer.jump(Side::Right, buffer.sec_cur());
            buffer.sec_cur_set(new_pos, true)
        }
        InputEvent::Resize(size) => *term_size = size,

        InputEvent::CtrlS => {
            let view = terminal_gui.additional_view_no_data();
            if let Some(active_view) = view {
                if active_view == gui::AdditionalViewNoData::Explorer {
                    rtn = AdditionalViewAction::Unset;
                } else {
                    rtn = AdditionalViewAction::SetTo(gui::AdditionalViewNoData::Explorer);
                }
            } else {
                rtn = AdditionalViewAction::SetTo(gui::AdditionalViewNoData::Explorer);
            }
        }

        InputEvent::CtrlD => {
            let view = terminal_gui.additional_view_no_data();
            if let Some(active_view) = view {
                if active_view == gui::AdditionalViewNoData::Table {
                    rtn = AdditionalViewAction::Unset;
                } else {
                    rtn = AdditionalViewAction::SetTo(gui::AdditionalViewNoData::Table);
                }
            } else {
                rtn = AdditionalViewAction::SetTo(gui::AdditionalViewNoData::Table);
            }
        }

        InputEvent::CtrlT => {
            let view = terminal_gui.additional_view_no_data();
            if let Some(active_view) = view {
                if active_view == gui::AdditionalViewNoData::Dropdown {
                    rtn = AdditionalViewAction::Unset;
                } else {
                    rtn = AdditionalViewAction::SetTo(gui::AdditionalViewNoData::Dropdown);
                }
            } else {
                rtn = AdditionalViewAction::SetTo(gui::AdditionalViewNoData::Dropdown);
            }
        }

        _ => ()
        // InputEvent::CtrlD => buffer.ctrl_d(),
        // InputEvent::CtrlS => buffer.ctrl_s(),
        // InputEvent::CtrlT => buffer.ctrl_t(),
        //
        // InputEvent::ArrowUp => buffer.up(),
        // InputEvent::ArrowDown => buffer.down(),
        //
        // InputEvent::ShiftArrowUp => buffer.shift_up(),
        // InputEvent::ShiftArrowDown => buffer.shift_down(),
        //
        //
        // InputEvent::Other(key) => buffer.other(key),
    }
    buffer.update();
    rtn
}

fn execute_action(action: gui::ActionToExecute, buffer: &mut buffer::InputBuffer) {
    match action {
        gui::ActionToExecute::SetClosestMatch(s) => {
            let curr_arg = buffer.get_curr_arg();
            buffer.set_closest_match_on_hint(curr_arg, s);
        }
    }
}

fn update_view(view: AdditionalViewAction, terminal_gui: &mut gui::terminal::TerminalGUI, write_from_line: u16) {
    match view {
        AdditionalViewAction::None => (),
        AdditionalViewAction::SetTo(view) => {
            terminal_gui.clear_output(write_from_line);
            terminal_gui.set_using(Some(view))
        }
        AdditionalViewAction::Unset => {
            terminal_gui.clear_output(write_from_line);
            terminal_gui.set_using(None)
        }
    }
}

#[allow(unused_variables)]
fn runtime_loop(
    program_state: &state::ProgramState,
    mut buffer: buffer::InputBuffer,
    mut terminal_gui: gui::terminal::TerminalGUI,
) {
    ansi::erase_screen();

    let mut term_size = (1, 1);

    let mut iter: u128 = 0;
    let mut positions;
    let mut write_from_line;
    let mut input;
    let mut action_to_take;
    loop {
        if iter == 1 {
            term_size = crossterm::terminal::size().unwrap();
        }

        input = match input::get_input() {
            Ok(inp) => inp,
            Err(_) => continue
        };
        if input == InputEvent::CtrlC {
            break;
        }

        positions = terminal_gui.calculate_increased_length(&buffer, term_size);
        write_from_line = positions.1.1 + 1;

        action_to_take = terminal_gui.action_before_write(
            &buffer,
            input.clone(),
            term_size,
            write_from_line,
            positions.0,
            positions.2
        );

        if let gui::ActionToTake::WriteBuffer(action) = action_to_take {
            if let gui::ActionType::Other(other) = action {
                execute_action(other, &mut buffer);
            } else {
                let view = update_buffer(input.clone(), &mut buffer, &mut term_size, &terminal_gui);
                update_view(view, &mut terminal_gui, write_from_line);

                terminal_gui.action_before_write(
                    &buffer,
                    InputEvent::Dummy,
                    term_size,
                    write_from_line,
                    positions.0,
                    positions.2,
                );
            }
        }

        positions = terminal_gui.calculate_increased_length(&buffer, term_size);

        terminal_gui.write_output(&buffer, input, term_size, positions.0);

        iter += 1;
    }
}

fn main() {
    let program_state = {
        let config = config::read_or_create_all_configs();

        let current_working_directory = std::env::current_dir().unwrap();

        state::ProgramState::init(config, current_working_directory)
    };
    let buffer = buffer::InputBuffer::init(&program_state);
    let terminal_gui = gui::terminal::TerminalGUI::init(&program_state);

    crossterm::terminal::enable_raw_mode().unwrap();
    runtime_loop(&program_state, buffer, terminal_gui);
    crossterm::terminal::disable_raw_mode().unwrap();
}
