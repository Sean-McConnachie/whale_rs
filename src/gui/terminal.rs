use crate::{state, utils, input, buffer, enums, ansi};
use crate::config::theme;

use super::GUITrait;

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

pub struct TerminalGUI<'a>
{
    program_state: &'a state::ProgramState,

    additional_view: Option<super::AdditionalView<'a>>,

    current_line: u16,
}

impl<'a> TerminalGUI<'a>
{
    pub fn init(program_state: &'a state::ProgramState) -> Self {
        Self {
            program_state,
            additional_view: None,
            current_line: 0,
        }
    }

    pub fn output_path(&self) {
        let short_cwd = utils::short_path(&self.program_state.current_working_directory);
        let theme = &self.program_state.config.theme;
        super::output_str(&theme.console_main.normal, &short_cwd);
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

        let theme = &self.program_state.config.theme;

        let (cur_a, cur_b) = buf.cursor_range();
        let arg_hints = buf.get_argument_hints();
        let splits = buf.get_splits();

        let (mut high_s, mut high_b) = if cur_a == cur_b {
            (HighlightState::AfterHighlight, false)
        } else if cur_a == 0 {
            (HighlightState::InHighlight, true)
        } else {
            (HighlightState::BeforeHighlight, false)
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
            match high_s {
                HighlightState::BeforeHighlight => {
                    if cur_a >= start && cur_a < stop {
                        handle_split_arg(&style, &arg, high_b, cur_a - start);
                        high_s.next();
                        high_b = !high_b;
                    } else {
                        handle_normal_arg(&style, &arg, high_b);
                    }
                }
                HighlightState::InHighlight => {
                    if cur_b >= start && cur_b < stop {
                        handle_split_arg(&style, &arg, high_b, cur_b - start);
                        high_s.next();
                        high_b = !high_b;
                    } else {
                        handle_normal_arg(&style, &arg, high_b);
                    }
                }
                HighlightState::AfterHighlight => {
                    handle_normal_arg(&style, &arg, high_b);
                }
            }
        }
    }

    pub fn write_output(
        &mut self,
        buf: &buffer::InputBuffer,
        event: input::InputEvent,
        term_size: super::TerminalXY,
    ) {
        ansi::erase_screen();
        ansi::move_to((0, 0));

        self.output_path();

        self.output_buffer(buf);

        if let Some(view) = &mut self.additional_view {
            view.write_output(event, term_size);
        }

        ansi::reset();
        ansi::flush();
    }

    pub fn action_on_buffer(&self, event: input::InputEvent) -> super::PropagateAction {
        if let Some(view) = &self.additional_view {
            return view.action_on_buffer(event);
        }
        true as super::PropagateAction
    }

    pub fn clear_output() -> () {}
}
