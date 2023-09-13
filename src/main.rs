use whale_rs::input::InputEvent as IEvent;

fn update_buffer(input: whale_rs::input::InputEvent, buffer: &mut whale_rs::buffer::InputBuffer) {
    match input {
        IEvent::Esc => buffer.unset_secondary_cursor(),
        IEvent::Backspace => buffer.del_n(whale_rs::buffer::Side::Left, 1),
        IEvent::Delete => buffer.del_n(whale_rs::buffer::Side::Right, 1),
        IEvent::Enter => {
            // TODO: Run
            // buffer.enter();
            // command_run = true;
        }
        IEvent::Tab => {
            // TODO: Tab
            // buffer.tab()
        }
        IEvent::Character(c) => buffer.insert_char_main_cursor(c),
        IEvent::CtrlBackspace => buffer.del_jump(whale_rs::buffer::Side::Left),
        IEvent::CtrlDelete => buffer.del_jump(whale_rs::buffer::Side::Left),
        _ => ()
        // IEvent::CtrlC => buffer.ctrl_c(),
        // IEvent::CtrlD => buffer.ctrl_d(),
        // IEvent::CtrlS => buffer.ctrl_s(),
        // IEvent::CtrlT => buffer.ctrl_t(),
        //
        // IEvent::ArrowUp => buffer.up(),
        // IEvent::ArrowDown => buffer.down(),
        // IEvent::ArrowLeft => buffer.left(),
        // IEvent::ArrowRight => buffer.right(),
        //
        // IEvent::CtrlArrowLeft => buffer.ctrl_left(),
        // IEvent::CtrlArrowRight => buffer.ctrl_right(),
        //
        // IEvent::ShiftArrowUp => buffer.shift_up(),
        // IEvent::ShiftArrowDown => buffer.shift_down(),
        // IEvent::ShiftArrowLeft => buffer.shift_left(),
        // IEvent::ShiftArrowRight => buffer.shift_right(),
        //
        // IEvent::CtrlShiftArrowLeft => buffer.ctrl_shift_left(),
        // IEvent::CtrlShiftArrowRight => buffer.ctrl_shift_right(),
        //
        // IEvent::Resize(size) => buffer.resize(size),
        // IEvent::Other(key) => buffer.other(key),
    }
    buffer.update();
}

fn runtime_loop(
    program_state: &whale_rs::state::ProgramState,
    mut buffer: whale_rs::buffer::InputBuffer,
    mut terminal_gui: whale_rs::gui::terminal::TerminalGUI,
) {
    let TODO_REMOVE = (0, 0);

    whale_rs::ansi::erase_screen();

    let mut input;
    let mut action_on_buffer;
    loop {
        input = match whale_rs::input::get_input() {
            Ok(inp) => inp,
            Err(_) => continue
        };
        if input == whale_rs::input::InputEvent::CtrlC {
            break;
        }

        action_on_buffer = terminal_gui.action_on_buffer(input.clone());
        if action_on_buffer {
            update_buffer(input.clone(), &mut buffer);
        }

        terminal_gui.write_output(&buffer, input, TODO_REMOVE);
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
    let terminal_gui = whale_rs::gui::terminal::TerminalGUI::init(&program_state);

    crossterm::terminal::enable_raw_mode().unwrap();
    runtime_loop(&program_state, buffer, terminal_gui);
    crossterm::terminal::disable_raw_mode().unwrap();
}
