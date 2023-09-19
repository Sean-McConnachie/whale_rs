use whale_rs::input::InputEvent as IEvent;
use whale_rs::buffer::Side;
use whale_rs::gui;

use whale_rs::gui::GUITrait;

enum AdditionalViewAction {
    None,
    SetTo(gui::AdditionalViewNoData),
    Unset,
}

fn update_buffer<'a>(
    input: whale_rs::input::InputEvent,
    buffer: &'a mut whale_rs::buffer::InputBuffer,
    term_size: &mut whale_rs::ansi::TerminalXY,
    terminal_gui: &whale_rs::gui::terminal::TerminalGUI,
) -> AdditionalViewAction {
    let mut rtn = AdditionalViewAction::None;
    match input {
        IEvent::Esc => buffer.unset_secondary_cursor(),
        IEvent::Backspace => buffer.del_n(Side::Left, 1),
        IEvent::Delete => buffer.del_n(Side::Right, 1),
        IEvent::Enter => {
            // TODO: Run
            // buffer.enter();
            // command_run = true;
        }
        IEvent::Tab => {
            let hints = buffer.get_argument_hints();
            let curr = buffer.arg_locs(buffer.get_curr_arg());
            let hint = &hints[buffer.get_curr_arg()].1;
            let hint_arg = hint.last_closest_match().unwrap()[(curr.1 - curr.0 - hint.disregard())..].to_string();
            buffer.insert_str_main_cursor(&hint_arg);
        }
        IEvent::Character(c) => {
            buffer.del_betw_curs();
            buffer.insert_char_main_cursor(c);
        }
        IEvent::CtrlBackspace => buffer.del_jump(Side::Left),
        IEvent::CtrlDelete => buffer.del_jump(Side::Left),
        IEvent::CtrlC => unreachable!("This should be handled outside of the match statement!"),

        IEvent::ArrowLeft => {
            buffer.main_cur_set(buffer.move_n(Side::Left, 1, buffer.main_cur()));
            buffer.unset_secondary_cursor();
        }
        IEvent::ArrowRight => {
            buffer.main_cur_set(buffer.move_n(Side::Right, 1, buffer.main_cur()));
            buffer.unset_secondary_cursor();
        }

        IEvent::CtrlArrowLeft => {
            buffer.main_cur_set(buffer.jump(Side::Left, buffer.main_cur()));
            buffer.unset_secondary_cursor();
        }
        IEvent::CtrlArrowRight => {
            buffer.main_cur_set(buffer.jump(Side::Right, buffer.main_cur()));
            buffer.unset_secondary_cursor();
        }

        IEvent::AltArrowLeft => {
            buffer.enable_sec_cur_if_not_active();
            let new_pos = buffer.move_n(Side::Left, 1, buffer.sec_cur());
            buffer.sec_cur_set(new_pos, true)
        }
        IEvent::AltArrowRight => {
            buffer.enable_sec_cur_if_not_active();
            let new_pos = buffer.move_n(Side::Right, 1, buffer.sec_cur());
            buffer.sec_cur_set(new_pos, true);
        }

        IEvent::CtrlShiftArrowLeft => {
            buffer.enable_sec_cur_if_not_active();
            let new_pos = buffer.jump(Side::Left, buffer.sec_cur());
            buffer.sec_cur_set(new_pos, true)
        }
        IEvent::CtrlShiftArrowRight => {
            buffer.enable_sec_cur_if_not_active();
            let new_pos = buffer.jump(Side::Right, buffer.sec_cur());
            buffer.sec_cur_set(new_pos, true)
        }
        IEvent::Resize(size) => *term_size = size,

        IEvent::CtrlD => {
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
        _ => ()
        // IEvent::CtrlD => buffer.ctrl_d(),
        // IEvent::CtrlS => buffer.ctrl_s(),
        // IEvent::CtrlT => buffer.ctrl_t(),
        //
        // IEvent::ArrowUp => buffer.up(),
        // IEvent::ArrowDown => buffer.down(),
        //
        // IEvent::ShiftArrowUp => buffer.shift_up(),
        // IEvent::ShiftArrowDown => buffer.shift_down(),
        //
        //
        // IEvent::Other(key) => buffer.other(key),
    }
    buffer.update();
    rtn
}

fn execute_action(action: gui::ActionToExecute, buffer: &mut whale_rs::buffer::InputBuffer) {
    match action {
        gui::ActionToExecute::None => (),
        gui::ActionToExecute::SetClosestMatch(s) => {
            let curr_arg = buffer.get_curr_arg();
            buffer.set_closest_match_on_hint(curr_arg, s);
        }
    }
}

fn runtime_loop(
    program_state: &whale_rs::state::ProgramState,
    mut buffer: whale_rs::buffer::InputBuffer,
    mut terminal_gui: gui::terminal::TerminalGUI,
) {
    whale_rs::ansi::erase_screen();

    let mut term_size = crossterm::terminal::size().unwrap();

    let mut input;
    let mut action_to_take;
    let mut action_to_execute;
    loop {
        input = match whale_rs::input::get_input() {
            Ok(inp) => inp,
            Err(_) => continue
        };
        if input == whale_rs::input::InputEvent::CtrlC {
            break;
        }

        action_to_take = terminal_gui.action_on_buffer(&buffer, input.clone());
         if action_to_take != gui::ActionToTake::BlockBuffer {
            let view = update_buffer(input.clone(), &mut buffer, &mut term_size, &terminal_gui);
            match view {
                AdditionalViewAction::None => (),
                AdditionalViewAction::SetTo(view) => {
                    terminal_gui.clear_output();
                    terminal_gui.set_using(Some(view))
                },
                AdditionalViewAction::Unset => {
                    terminal_gui.clear_output();
                    terminal_gui.set_using(None)
                },
            }
        }

        action_to_execute = terminal_gui.write_output(&buffer, input, term_size);
        if action_to_execute != gui::ActionToExecute::None {}
    }
}

fn main() {
    let program_state = {
        let config = whale_rs::config::read_or_create_all_configs();
        if !config.core.data_dir.exists() {
            std::fs::create_dir_all(&config.core.data_dir).unwrap();
        }

        let current_working_directory = std::env::current_dir().unwrap();

        whale_rs::state::ProgramState::init(config, current_working_directory)
    };
    let buffer = whale_rs::buffer::InputBuffer::init(&program_state);
    let terminal_gui = gui::terminal::TerminalGUI::init(&program_state);

    crossterm::terminal::enable_raw_mode().unwrap();
    runtime_loop(&program_state, buffer, terminal_gui);
    crossterm::terminal::disable_raw_mode().unwrap();
}
