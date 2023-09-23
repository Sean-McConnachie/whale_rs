use crate::{ansi, buffer, state};
use crate::ansi::TerminalXY;
use crate::gui::{ActionToExecute, ActionToTake, ActionType, HighlightDrawn};
use crate::gui::terminal::CursorPos;
use crate::input::InputEvent;

pub struct DropdownGUI<'a> {
    program_state: &'a state::ProgramState,

    cursor_pos: usize,
    table_scroll: usize,

    // Temporary variables used to transition between functions
    prev_len: usize,
    hints_iterator: Vec<usize>,

    arg_start: CursorPos,
}

impl<'a> DropdownGUI<'a> {
    #[inline(always)]
    fn arrow_up(&mut self, cur_max: usize, scroll_max: usize) -> bool {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        } else if self.table_scroll > 0 {
            self.table_scroll -= 1;
        } else {
            self.table_scroll = scroll_max;
            self.cursor_pos = cur_max;
        }
        true
    }

    #[inline(always)]
    fn arrow_down(&mut self, cur_max: usize, scroll_max: usize) -> bool {
        if self.cursor_pos < cur_max {
            self.cursor_pos += 1;
        } else if self.table_scroll < scroll_max {
            self.table_scroll += 1;
        } else {
            self.table_scroll = 0;
            self.cursor_pos = 0;
        }
        true
    }
}

impl<'a> super::GUITrait<'a> for DropdownGUI<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self {
        Self {
            program_state,
            cursor_pos: 0,
            table_scroll: 0,
            prev_len: 0,
            hints_iterator: Vec::new(),
            arg_start: (0, 0),
        }
    }

    #[allow(unused_variables)]
    fn action_before_write(
        &mut self,
        event: InputEvent,
        buffer: &buffer::InputBuffer,
        term_size: TerminalXY,
        write_from_line: u16,
        cursor_pos: CursorPos,
        arg_pos: CursorPos,
    ) -> ActionToTake {
        fn ceil_div(a: usize, b: usize) -> usize {
            (a + b - 1) / b
        }

        // Basic dimensions
        let write_table_from_line = (write_from_line + 1).min(term_size.1);
        let dropdown_rows = (term_size.1 - write_table_from_line)
            .min(self.program_state.config.gui.dropdown.max_rows);

        // Find relevant hints
        // TODO: This is a bit of a mess
        let mut disregard = 0;
        let mut arg = String::new();
        let mut hint: &[String] = &[];
        if let Some(h) = buffer.get_curr_hint_safe() {
            arg = h.0;
            hint = h.1.get_selection();
            disregard = h.1.disregard();
        };

        self.arg_start = (arg_pos.0 + disregard as u16, arg_pos.1);

        let mut hint_indexes = Vec::with_capacity(dropdown_rows as usize);
        for (i, s) in hint.iter().enumerate() {
            if s.starts_with(&arg[disregard..]) {
                hint_indexes.push(i);
            }
        }

        let num_hints = hint_indexes.len();

        // Reset dropdown if hints have changed
        if self.prev_len != num_hints {
            self.prev_len = num_hints;
            self.cursor_pos = 0;
            self.table_scroll = 0;
        }

        let scroll_max = ceil_div(num_hints, dropdown_rows as usize).saturating_sub(1);
        let cur_max = (dropdown_rows as usize).min(num_hints).saturating_sub(1);

        // Move cursor
        let should_set_closest = match event {
            InputEvent::ArrowUp => self.arrow_up(cur_max, scroll_max),
            InputEvent::ArrowDown => self.arrow_down(cur_max, scroll_max),
            _ => false
        };

        // Do some work for the next write_output stage
        self.hints_iterator = {
            let upper = (cur_max + 1).min(num_hints);
            (0..upper)
                .map(|x| {
                    hint_indexes[(x + self.table_scroll).rem_euclid(num_hints)]
                }).collect()
        };
        let ind = self.cursor_pos + self.table_scroll;

        // Return action
        if should_set_closest {
            let hint_ind = hint_indexes[ind];
            let hint = hint[hint_ind].clone();
            ActionToTake::WriteBuffer(ActionType::Other(ActionToExecute::SetClosestMatch(hint)))
        } else {
            ActionToTake::WriteBuffer(ActionType::Standard)
        }
    }

    #[allow(unused_variables)]
    fn write_output(
        &mut self,
        event: InputEvent,
        term_size: TerminalXY,
        write_from_line: u16,
        buf: &buffer::InputBuffer,
    ) {
        ansi::move_to((0, write_from_line));
        ansi::erase_line();

        fn multi_line_str(s: &str, max_len: usize) -> Vec<&str> {
            let mut lines = vec![];
            let max_ind = max_len.min(s.len());
            lines.push(&s[..max_ind]);
            if s.len() >= max_len {
                lines.extend(multi_line_str(&s[..max_len], max_len));
            }
            lines
        }

        {
            let mut hints: &[String] = &[];
            if let Some(hint) = buf.get_curr_hint_safe() {
                hints = hint.1.get_selection();
            }

            let mut cursor_drawn = HighlightDrawn::Before;
            let mut style = &self.program_state.config.theme.console_secondary.normal;
            let max_len = (term_size.0 - self.arg_start.0) as usize;
            let max_lines = (term_size.1 - self.arg_start.1) as usize;
            let mut num_lines = 1;
            for (i, ind) in self.hints_iterator.iter().enumerate() {
                let item = &hints[*ind];

                if cursor_drawn == HighlightDrawn::Before && i == self.cursor_pos {
                    cursor_drawn = HighlightDrawn::During;
                    style = &self.program_state.config.theme.console_secondary.highlighted;
                } else if cursor_drawn == HighlightDrawn::During {
                    cursor_drawn = HighlightDrawn::After;
                    style = &self.program_state.config.theme.console_secondary.normal;
                };

                let lines = multi_line_str(&item, max_len);
                for l in lines.into_iter() {
                    if num_lines >= max_lines {
                        break;
                    }
                    ansi::move_to((self.arg_start.0, self.arg_start.1 + num_lines as u16));
                    super::output_str(style, format!("{}", l).as_str());
                    num_lines += 1;
                }
            }
        }
    }

    #[allow(unused_variables)]
    fn clear_output(&mut self, write_from_line: u16) -> () {}
}
