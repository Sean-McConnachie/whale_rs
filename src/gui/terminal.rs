use crate::{state, utils, input, buffer, enums, ansi};

use super::GUITrait;

pub struct TerminalGUI<'a>
{
    program_state: &'a state::ProgramState,

    additional_view: Option<super::AdditionalView<'a>>,
}

impl<'a> TerminalGUI<'a>
{
    pub fn init(program_state: &'a state::ProgramState) -> Self {
        Self { program_state, additional_view: None }
    }

    pub fn output_path(&self) {
        let short_cwd = utils::short_path(&self.program_state.current_working_directory);
        let theme = &self.program_state.config.theme;
        super::output_str(&theme.console_main.normal, &short_cwd);
    }

    fn output_buffer(&self, buf: &buffer::InputBuffer) {
        let theme = &self.program_state.config.theme;

        let (cur_a, cur_b) = buf.cursor_range();
        let arg_hints = buf.get_argument_hints();

        for ((arg_type, hint), loc) in buf
            .arg_locs_iterator()
            .enumerate()
            .map(|(i, loc)|
                (&arg_hints[i], loc)) {
            let arg = buf.get_buffer_str(loc);
            let style = match arg_type {
                enums::ArgType::Executable => &theme.executable,
                enums::ArgType::Path => &theme.path,
                enums::ArgType::Text => &theme.text
            };
            super::output_str(&style.normal, &arg);
        }
    }

    pub fn write_output(
        &mut self,
        buf: &buffer::InputBuffer,
        event: input::InputEvent,
        term_size: super::TerminalXY,
    ) {
        ansi::erase_screen();
        ansi::move_to(1, 1);

        self.output_path();

        self.output_buffer(buf);

        if let Some(view) = &mut self.additional_view {
            view.write_output(event, term_size);
        }

        ansi::flush();
    }

    pub fn action_on_buffer(&self, event: input::InputEvent, ) -> super::PropagateAction {
        if let Some(view) = &self.additional_view {
            return view.action_on_buffer(event);
        }
        true as super::PropagateAction
    }

    pub fn clear_output() -> () {}
}
