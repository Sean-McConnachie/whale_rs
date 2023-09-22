use crate::{state, utils, input, buffer, enums, ansi, hints};
use crate::ansi::TerminalXY;
use crate::config::theme;
use crate::gui::{ActionToExecute, ActionToTake, ActionType};

use super::GUITrait;

// TODO: Fix inlay hints
// TODO: Add ConfigCommand structs to executables for hints

pub type CursorPos = (u16, u16);
pub type ArgPos = (u16, u16);

#[derive(Debug, PartialEq)]
enum HighlightState {
    BeforeHighlight,
    InHighlight,
    AfterHighlight,
}

impl HighlightState {
    fn next(&mut self) {
        match self {
            Self::BeforeHighlight => *self = Self::InHighlight,
            Self::InHighlight => *self = Self::AfterHighlight,
            Self::AfterHighlight => panic!("Cannot call next() on AfterHighlight"),
        }
    }
}

pub struct TerminalGUI<'a> {
    program_state: &'a state::ProgramState,

    additional_view: Option<super::AdditionalView<'a>>,

    current_line: u16,

    short_cwd: String,
}


impl<'a> TerminalGUI<'a>
{
    pub fn init(program_state: &'a state::ProgramState) -> Self {
        Self {
            short_cwd: String::new(),
            program_state,
            additional_view: None,
            current_line: 0,
        }
    }

    pub fn set_using(&mut self, view: Option<super::AdditionalViewNoData>) {
        if let Some(view) = view {
            self.additional_view = Some(super::AdditionalView::from_enum(view, self.program_state));
        } else {
            self.additional_view = None;
        }
    }

    pub fn additional_view_no_data(&self) -> Option<super::AdditionalViewNoData> {
        match &self.additional_view {
            Some(view) => Some(view.additional_view_no_data()),
            None => None,
        }
    }

    /// short_cwd is updated by calculate_increased_length
    pub fn output_path(&self) {
        let theme = &self.program_state.config.theme;
        super::output_str(&theme.console_main.normal, &self.short_cwd);
    }

    fn output_buffer(&self, buf: &buffer::InputBuffer) {
        if buf.len() == 0 { return; }

        fn handle_normal_arg(style: &theme::StylePair, arg: &str, highlighted: bool) {
            match highlighted {
                true => super::output_str(&style.highlighted, arg),
                false => super::output_str(&style.normal, arg),
            }
        }

        fn handle_split_arg(style: &theme::StylePair, arg: &str, mut highlighted: bool, split_at: usize) {
            let (a, b) = arg.split_at(split_at);
            match highlighted {
                true => super::output_str(&style.highlighted, a),
                false => super::output_str(&style.normal, a),
            }
            highlighted = !highlighted;
            match highlighted {
                true => super::output_str(&style.highlighted, b),
                false => super::output_str(&style.normal, b),
            }
        }

        fn handle_suggestion_arg(
            style: &theme::StylePair,
            arg: &str,
            cur_a: usize,
            hint_style: &theme::Style,
            hint: &hints::Hint,
        ) {
            let disregard = hint.disregard();
            if let Some(hint) = hint.last_closest_match() {
                super::output_str(&style.normal, &arg[..cur_a]);
                super::output_str(&hint_style, &hint[(cur_a - disregard)..]);
                super::output_str(&style.normal, &arg[cur_a..]);
            } else {
                super::output_str(&style.normal, &arg);
            }
        }

        let theme = &self.program_state.config.theme;

        // TODO: Increased length by inline hint
        let mut increased_length = 0;
        let (cur_a, cur_b) = buf.cursor_range();
        let arg_hints = buf.get_argument_hints();
        let splits = buf.get_splits();

        let (mut hilt_ste, mut hilt_curr, hilt_active) = if cur_a == cur_b {
            (HighlightState::AfterHighlight, false, false)
        } else if cur_a == 0 {
            (HighlightState::InHighlight, true, true)
        } else {
            (HighlightState::BeforeHighlight, false, true)
        };

        for i in 0..splits.len() - 1 {
            let locs = (splits[i], splits[i + 1]);
            let style = match i % 2 == 1 {
                true => &theme.text,
                false => {
                    let arg = &arg_hints[i / 2];
                    match arg.0 {
                        enums::ArgType::Executable => &theme.executable,
                        enums::ArgType::Path => &theme.path,
                        enums::ArgType::Text => &theme.text
                    }
                }
            };
            let arg = buf.get_buffer_str(locs);

            let (start, stop) = locs;
            match hilt_ste {
                HighlightState::BeforeHighlight => {
                    if cur_a >= start && cur_a < stop {
                        handle_split_arg(&style, &arg, hilt_curr, cur_a - start);
                        hilt_ste.next();
                        hilt_curr = !hilt_curr;
                    } else {
                        handle_normal_arg(&style, &arg, hilt_curr);
                    }
                }
                HighlightState::InHighlight => {
                    if cur_b >= start && cur_b < stop {
                        handle_split_arg(&style, &arg, hilt_curr, cur_b - start);
                        hilt_ste.next();
                        hilt_curr = !hilt_curr;
                    } else {
                        handle_normal_arg(&style, &arg, hilt_curr);
                    }
                }
                HighlightState::AfterHighlight => {
                    // We do not show inline hints when highlighted
                    if !hilt_active && cur_a >= start && cur_b <= stop {
                        handle_suggestion_arg(
                            &style,
                            &arg,
                            cur_a - start,
                            &theme.console_secondary.normal,
                            &arg_hints[i / 2].1);
                    } else {
                        handle_normal_arg(&style, &arg, hilt_curr);
                    }
                }
            }
        }
    }

    pub fn action_before_write(
        &mut self,
        buf: &buffer::InputBuffer,
        event: input::InputEvent,
        term_size: TerminalXY,
        write_from_line: u16,
        cursor_pos: CursorPos,
        arg_pos: CursorPos,
    ) -> ActionToTake {
        if event == input::InputEvent::Tab {
            let hints = buf.get_argument_hints();
            if !hints.is_empty() {
                // TODO: This is not a good way to do this
                if buf.get_curr_arg() >= hints.len() {
                    return ActionToTake::BlockBuffer;
                }
                return if hints[buf.get_curr_arg()].1.last_closest_match().is_some() {
                    ActionToTake::WriteBuffer(ActionType::Standard)
                } else {
                    ActionToTake::BlockBuffer
                };
            }
        }
        if let Some(view) = &mut self.additional_view {
            return view.action_before_write(event, &buf, term_size, write_from_line, cursor_pos, arg_pos);
        }
        ActionToTake::WriteBuffer(ActionType::Standard)
    }

    pub fn write_output(
        &mut self,
        buf: &buffer::InputBuffer,
        event: input::InputEvent,
        term_size: TerminalXY,
        cursor_position: CursorPos,
    ) {
        ansi::erase_screen();
        ansi::move_to((0, self.current_line));

        self.output_path();
        self.output_buffer(buf);

        if let Some(view) = &mut self.additional_view {
            view.write_output(event, term_size, self.current_line + cursor_position.1 + 1, buf)
        } else {};

        ansi::move_to((cursor_position.0, self.current_line + cursor_position.1));

        ansi::reset();
        ansi::flush();
    }

    pub fn calculate_increased_length(
        &mut self,
        buffer: &buffer::InputBuffer,
        term_size: TerminalXY,
    ) -> (CursorPos, TerminalXY, ArgPos) {
        fn pos_to_xy(pos: u16, term_size: TerminalXY) -> (u16, u16) {
            // TODO: This shouldn't really equal 0 at any point
            if term_size.0 == 0 {
                (0, 0)
            } else {
                (pos % term_size.0, pos / term_size.0)
            }
        }

        self.short_cwd = utils::short_path(&self.program_state.current_working_directory);

        let upto_end = buffer.len() + self.short_cwd.len();
        let upto_cursor = buffer.main_cur().position() + self.short_cwd.len();
        let upto_curr_arg = {
            if buffer.num_args() == 0 {
                0 + self.short_cwd.len()
            } else {
                let mut ind = buffer.get_curr_arg() * 2;
                if ind >= buffer.get_splits().len() {
                    ind -= 2;
                }
                buffer.get_splits()[ind] + self.short_cwd.len()
            }
        };

        (
            pos_to_xy(upto_cursor as u16, term_size),
            pos_to_xy(upto_end as u16, term_size),
            pos_to_xy(upto_curr_arg as u16, term_size)
        )
    }

    pub fn clear_output(&mut self, write_from_line: u16) {
        if let Some(view) = &mut self.additional_view {
            view.clear_output(write_from_line);
        }
    }
}
