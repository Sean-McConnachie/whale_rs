use std::cell::RefCell;
use std::rc::Rc;
use crate::{ansi, buffer, state};
use crate::ansi::TerminalXY;
use crate::gui::{ActionToExecute, ActionToTake, ActionType, HighlightDrawn, ViewType};
use crate::gui::terminal::CursorPos;
use crate::input::InputEvent;

pub struct TableGUI {
    program_state: Rc<RefCell<state::ProgramState>>,

    cursor_pos: TerminalXY,
    table_scroll: usize,

    // Temporary variables used to transition between functions
    prev_len: usize,
    grid_slots: TerminalXY,
    preceding_cur: usize,
    succeeding_cur: usize,
    hints_iterator: Vec<usize>,
}

impl TableGUI {
    const CURSOR_HOME: TerminalXY = (0, 0);
    const TABLE_SCROLL: usize = 0;

    pub fn init(program_state: Rc<RefCell<state::ProgramState>>) -> Self {
        Self {
            program_state,
            cursor_pos: Self::CURSOR_HOME,
            table_scroll: Self::TABLE_SCROLL,
            prev_len: 0,
            hints_iterator: Vec::new(),
            grid_slots: Self::CURSOR_HOME,
            preceding_cur: 0,
            succeeding_cur: 0,
        }
    }

    #[inline(always)]
    fn arrow_up(&mut self, cur_pos_last: TerminalXY, scroll_max: usize) -> bool {
        if self.cursor_pos.1 > 0 {
            self.cursor_pos.1 -= 1;
        } else if self.table_scroll > 0 {
            self.table_scroll -= 1;
        } else {
            self.cursor_pos.1 = cur_pos_last.1;
            self.table_scroll = scroll_max;
            if self.cursor_pos.0 > cur_pos_last.0 {
                self.cursor_pos.0 = cur_pos_last.0;
            }
        }
        true
    }

    #[inline(always)]
    fn arrow_down(&mut self, cur_pos_last: TerminalXY, scroll_max: usize, scroll_is_max: bool) -> bool {
        if !scroll_is_max {
            if self.cursor_pos.1 < cur_pos_last.1 {
                self.cursor_pos.1 += 1;
            } else if self.table_scroll == scroll_max.saturating_sub(1) {
                self.table_scroll += 1;
                self.cursor_pos.0 = cur_pos_last.0;
            } else {
                self.table_scroll += 1;
            }
        } else {
            if self.cursor_pos.1 == cur_pos_last.1.saturating_sub(1) {
                self.cursor_pos.1 = cur_pos_last.1;
                if self.cursor_pos.0 > cur_pos_last.0 {
                    self.cursor_pos.0 = cur_pos_last.0;
                }
            } else if self.cursor_pos.1 < cur_pos_last.1 {
                self.cursor_pos.1 += 1;
            } else {
                self.cursor_pos.1 = 0;
                self.table_scroll = 0;
            }
        }
        true
    }

    #[inline(always)]
    fn arrow_left(&mut self, cur_pos_last: TerminalXY, scroll_max: usize, grid_slots: TerminalXY) -> bool {
        if self.cursor_pos.0 > 0 {
            self.cursor_pos.0 -= 1;
        } else if self.table_scroll == scroll_max && cur_pos_last.1 == self.cursor_pos.1 {
            self.cursor_pos.0 = cur_pos_last.0;
        } else {
            self.cursor_pos.0 = grid_slots.0 - 1;
        }
        true
    }

    #[inline(always)]
    fn arrow_right(&mut self, cur_pos_last: TerminalXY, scroll_max: usize, grid_slots: TerminalXY) -> bool {
        if self.cursor_pos.0 == cur_pos_last.0
            && self.table_scroll == scroll_max
            && cur_pos_last.1 == self.cursor_pos.1 {
            self.cursor_pos.0 = 0;
        } else if self.cursor_pos.0 < grid_slots.0 - 1 {
            self.cursor_pos.0 += 1;
        } else {
            self.cursor_pos.0 = 0;
        }
        true
    }
}

impl super::GUITrait for TableGUI {
    fn view_type(&self) -> ViewType {
        ViewType::Table
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
        // Basic dimensions
        let write_table_from_line = (write_from_line + 1).min(term_size.1);
        self.grid_slots = (
            term_size.0 / self.program_state.borrow().config.gui.table.max_field_len,
            term_size.1 - write_table_from_line
        ) as TerminalXY;
        let grid_slots = self.grid_slots;
        let grid_num_slots = (grid_slots.0 * grid_slots.1) as usize;

        fn ceil_div(a: usize, b: usize) -> usize {
            (a + b - 1) / b
        }
        fn mod_sub_1(a: usize, n: usize) -> usize {
            let r = a % n;
            if r == 0 { n - 1 } else { r - 1 }
        }

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

        let mut hint_indexes = Vec::with_capacity(grid_num_slots);
        for (i, s) in hint.iter().enumerate() {
            if s.starts_with(&arg[disregard..]) {
                hint_indexes.push(i);
            }
        }

        let num_hints = hint_indexes.len();

        // Reset table if hints have changed
        if self.prev_len != num_hints {
            self.prev_len = num_hints;
            self.cursor_pos = Self::CURSOR_HOME;
            self.table_scroll = Self::TABLE_SCROLL;
        }

        // This is the last position in the grid given the current number of hints and scroll
        let cur_pos_last = (
            (mod_sub_1(num_hints, grid_slots.0 as usize) as u16),
            (ceil_div(num_hints, grid_slots.0 as usize) as u16)
                .min(grid_slots.1).saturating_sub(1)
        );

        let scroll_max = ceil_div(num_hints, grid_slots.0 as usize).saturating_sub(grid_slots.1 as usize);
        let mut scroll_is_max = self.table_scroll == scroll_max;

        // Move cursor
        let should_set_closest = match event {
            InputEvent::ArrowUp => self.arrow_up(cur_pos_last, scroll_max),
            InputEvent::ArrowDown => self.arrow_down(cur_pos_last, scroll_max, scroll_is_max),
            InputEvent::ArrowLeft => self.arrow_left(cur_pos_last, scroll_max, grid_slots),
            InputEvent::ArrowRight => self.arrow_right(cur_pos_last, scroll_max, grid_slots),
            _ => false
        };

        scroll_is_max = self.table_scroll == scroll_max;

        // Do some work for the next write_output stage
        self.hints_iterator = {
            let upper = match (scroll_is_max, num_hints < grid_num_slots) {
                (false, false) => grid_num_slots,
                (false, true) => num_hints,
                (true, false) => {
                    let items_mod_slots = num_hints % grid_slots.0 as usize;
                    grid_num_slots - (grid_slots.0 as usize - items_mod_slots)
                }
                (true, true) => num_hints,
            };

            (0..upper)
                .map(|x| {
                    let i = (x + (self.table_scroll * grid_slots.0 as usize))
                        .rem_euclid(num_hints);
                    hint_indexes[i]
                }).collect()
        };
        let ind = (self.cursor_pos.1 as usize + self.table_scroll)
            * grid_slots.0 as usize + self.cursor_pos.0 as usize;
        self.preceding_cur = ind;
        self.succeeding_cur = (num_hints - ind).saturating_sub(1);

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

        pub type AddDots = bool;
        fn shorten_str(s: &str, max_len: usize) -> (AddDots, &str) {
            // shorten with ...
            if s.len() > max_len {
                (true, &s[..max_len - 3])
            } else {
                (false, s)
            }
        }

        let program_state = self.program_state.borrow();

        {
            let style = &program_state.config.theme.console_main.normal;
            let s = format!("{} ^ / v {}", self.preceding_cur, self.succeeding_cur);
            super::output_str(style, s.as_str());
            ansi::move_down(1);
            ansi::move_to_column(0);
            ansi::erase_line();
        }

        {
            let mut hints: &[String] = &[];
            if let Some(hint) = buf.get_curr_hint_safe() {
                hints = hint.1.get_selection();
            }

            let max_len = program_state.config.gui.table.max_field_len as usize;
            let mut first = true;
            let mut cursor_drawn = HighlightDrawn::Before;
            let mut row = 0;
            let mut col = 0;
            let mut style = &program_state.config.theme.console_secondary.normal;

            for i in &self.hints_iterator {
                let item = &hints[*i];

                if col % self.grid_slots.0 == 0 && !first {
                    ansi::move_down(1);
                    ansi::move_to_column(0);
                    ansi::erase_line();
                    row += 1;
                    col = 0;
                }
                first = false;

                if cursor_drawn == HighlightDrawn::Before
                    && row == self.cursor_pos.1 && col == self.cursor_pos.0 {
                    cursor_drawn = HighlightDrawn::During;
                    style = &program_state.config.theme.console_secondary.highlighted;
                } else if cursor_drawn == HighlightDrawn::During {
                    cursor_drawn = HighlightDrawn::After;
                    style = &program_state.config.theme.console_secondary.normal;
                };

                let l = match shorten_str(item, max_len) {
                    (true, s) => {
                        super::output_str(style, format!("{}...", s).as_str());
                        s.len() + 3
                    }
                    (false, s) => {
                        super::output_str(style, format!("{}", s).as_str());
                        s.len()
                    }
                };
                for _ in 0..(max_len - l) {
                    print!(" ");
                }
                col += 1;
            }
        }
    }

    fn clear_output(&mut self, write_from_line: u16) {
        ansi::move_to((0, write_from_line));
        ansi::erase_screen_from_cursor();
    }
}