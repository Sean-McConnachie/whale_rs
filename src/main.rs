use std::cell::RefCell;
use std::rc::Rc;
use whale_rs::input::{InputEvent};
use whale_rs::buffer::Side;
use whale_rs::{ansi, buffer, config, execution, gui, input, parser, state};
use whale_rs::gui::{explorer, GUITrait, ViewType};

fn toggle_view_action(
    view_action: &mut AdditionalViewAction,
    active_view: Option<ViewType>,
    target: ViewType) {
    if let Some(active) = active_view {
        if active == target {
            *view_action = AdditionalViewAction::Unset;
        } else {
            *view_action = AdditionalViewAction::SetTo(target);
        }
    } else {
        *view_action = AdditionalViewAction::SetTo(target);
    }
}

enum AdditionalViewAction {
    None,
    SetTo(ViewType),
    Unset,
}

fn update_buffer(
    input: InputEvent,
    program_state: &Rc<RefCell<state::ProgramState>>,
    buffer: &mut buffer::InputBuffer,
    term_size: &mut ansi::TerminalXY,
    terminal_gui: &mut gui::terminal::TerminalGUI,
    arg_parser: &mut parser::ArgumentParser,
) -> AdditionalViewAction {
    let mut rtn = AdditionalViewAction::None;
    match input {
        InputEvent::Esc => buffer.unset_secondary_cursor(),
        InputEvent::Backspace => buffer.del_n(Side::Left, 1),
        InputEvent::Delete => buffer.del_n(Side::Right, 1),
        InputEvent::Enter => {
            let (new_line, _status) = execution::running::run_command(
                program_state.clone(),
                buffer,
                arg_parser,
            );
            if let Some(line) = new_line {
                terminal_gui.set_current_line(line);
            }
            buffer.clear_all();
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
            let view = terminal_gui.view_type();
            toggle_view_action(&mut rtn, view, ViewType::Explorer)
        }

        InputEvent::CtrlD => {
            let view = terminal_gui.view_type();
            toggle_view_action(&mut rtn, view, ViewType::Dropdown)
        }

        InputEvent::CtrlT => {
            let view = terminal_gui.view_type();
            toggle_view_action(&mut rtn, view, ViewType::Table)
        }

        _ => ()
        // TODO: History
    }

    buffer.update();
    arg_parser.reinit(buffer.first_arg());
    buffer.update_arguments(arg_parser);
    rtn
}

fn execute_action(
    action: gui::ActionToExecute,
    buffer: &mut buffer::InputBuffer,
    arg_parser: &mut parser::ArgumentParser) {
    match action {
        gui::ActionToExecute::SetClosestMatch(s) => {
            let curr_arg = buffer.get_curr_arg();
            buffer.set_closest_match_on_hint(curr_arg, s);
        }
        gui::ActionToExecute::SetBuffer(s) => {
            buffer.clear_all();
            buffer.insert_str_main_cursor(&s);
            buffer.update();
            buffer.update_arguments(arg_parser);
        }
    }
}

fn update_view(
    view: AdditionalViewAction,
    terminal_gui: &mut gui::terminal::TerminalGUI,
    write_from_line: u16,
    program_state: &Rc<RefCell<state::ProgramState>>,
) {
    match view {
        AdditionalViewAction::None => (),
        AdditionalViewAction::Unset => {
            terminal_gui.clear_output(write_from_line);
            terminal_gui.set_using(None)
        }
        AdditionalViewAction::SetTo(view) => {
            terminal_gui.clear_output(write_from_line);
            let trait_obj = match view {
                ViewType::Table => {
                    let table = gui::table::TableGUI::init(program_state.clone());
                    Box::new(table) as Box<dyn GUITrait>
                }
                ViewType::Dropdown => {
                    let dropdown = gui::dropdown::DropdownGUI::init(program_state.clone());
                    Box::new(dropdown) as Box<dyn GUITrait>
                }
                ViewType::Explorer => {
                    let explorer = explorer::FileExplorerGUI::init(program_state.clone());
                    Box::new(explorer) as Box<dyn GUITrait>
                }
            };
            terminal_gui.set_using(Some(trait_obj));
        }
    }
}

#[allow(unused_variables)]
fn runtime_loop(
    program_state: Rc<RefCell<state::ProgramState>>,
    mut buffer: buffer::InputBuffer,
    mut terminal_gui: gui::terminal::TerminalGUI,
    mut argument_parser: parser::ArgumentParser,
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
            ansi::move_down(1);
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
            positions.2,
        );

        if let gui::ActionToTake::WriteBuffer(action) = action_to_take {
            if let gui::ActionType::Other(other) = action {
                execute_action(other, &mut buffer, &mut argument_parser);
            } else {
                let view = update_buffer(
                    input.clone(),
                    &program_state,
                    &mut buffer,
                    &mut term_size,
                    &mut terminal_gui,
                    &mut argument_parser,
                );
                update_view(view, &mut terminal_gui, write_from_line, &program_state);

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
        let default_shell = config.core.default_shell.clone();

        state::ProgramState::init(config, current_working_directory, default_shell)
    };
    let program_state = Rc::new(RefCell::new(program_state));
    let argument_parser = parser::ArgumentParser::new(program_state.clone());
    let buffer = buffer::InputBuffer::init(program_state.clone());
    let terminal_gui = gui::terminal::TerminalGUI::init(program_state.clone());

    crossterm::terminal::enable_raw_mode().unwrap();
    runtime_loop(program_state, buffer, terminal_gui, argument_parser);
    crossterm::terminal::disable_raw_mode().unwrap();
}
