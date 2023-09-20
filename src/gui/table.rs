use std::cmp::max;
use crate::{ansi, buffer, state};
use crate::ansi::TerminalXY;
use crate::gui::{ActionToExecute, ActionToTake};
use crate::input::InputEvent;

#[derive(PartialEq)]
enum HighlightDrawn {
    Before,
    During,
    After,
}

pub struct TableGUI<'a> {
    program_state: &'a state::ProgramState,

    cursor_pos: TerminalXY,
    table_scroll: usize,

    prev_len: usize,
}

impl<'a> TableGUI<'a> {
    const CURSOR_HOME: TerminalXY = (0, 0);
    const TABLE_SCROLL: usize = 0;
}

impl<'a> super::GUITrait<'a> for TableGUI<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self {
        Self {
            program_state,
            cursor_pos: Self::CURSOR_HOME,
            table_scroll: Self::TABLE_SCROLL,
            prev_len: 0,
        }
    }

    fn write_output(
        &mut self,
        event: InputEvent,
        term_size: TerminalXY,
        write_from_line: u16,
        buf: &buffer::InputBuffer,
    ) -> ActionToExecute {
        // TODO FIXME
        let term_size = (85, 6);

        let write_table_from_line = (write_from_line + 1).min(term_size.1);
        ansi::move_to((0, write_from_line));
        ansi::erase_line();

        let curr_arg = buf.get_curr_arg();
        let arg = if buf.num_args() > 0 {
            buf.get_buffer_str(buf.arg_locs(curr_arg))
        } else {
            String::new()
        };
        let (_arg_type, hint) = &buf.get_argument_hints()[curr_arg];
        let items = hint.get_selection().iter().filter(|s| s.starts_with(&arg)).collect::<Vec<_>>();
        if self.prev_len != items.len() {
            self.prev_len = items.len();
            self.cursor_pos = Self::CURSOR_HOME;
            self.table_scroll = Self::TABLE_SCROLL;
        }



        let grid_slots = (
            term_size.0 / self.program_state.config.gui.table.max_field_len,
            term_size.1 - write_table_from_line
        );

        fn ceil_div(a: usize, b: usize) -> usize {
            (a + b - 1) / b
        }

        fn mod_sub_1(a: usize, n: usize) -> usize {
            let r = a % n;
            if r == 0 {
                n - 1
            } else {
                r - 1
            }
        }

        let max_cursor_pos_y = (ceil_div(items.len(), grid_slots.0 as usize) as u16).min(grid_slots.1).saturating_sub(1);
        let max_scroll = (ceil_div(items.len(), grid_slots.0 as usize) as i64 - grid_slots.1 as i64).max(0) as usize;
        let total_slots = (grid_slots.0 * grid_slots.1) as usize;
        let bottom_cursor_x = (mod_sub_1(items.len(), grid_slots.0 as usize) as u16);
        let bottom_visible = self.table_scroll == max_scroll;

        match event {
            InputEvent::ArrowUp => {
                if self.cursor_pos.1 > 0 {
                    self.cursor_pos.1 -= 1;
                } else if self.table_scroll > 0 {
                    self.table_scroll -= 1;
                } else {
                    self.cursor_pos.1 = max_cursor_pos_y;
                    self.table_scroll = max_scroll;
                    if self.cursor_pos.0 > bottom_cursor_x {
                        self.cursor_pos.0 = bottom_cursor_x;
                    }
                }
            }

            InputEvent::ArrowDown => {
                if !bottom_visible {
                    if self.cursor_pos.1 < max_cursor_pos_y {
                        self.cursor_pos.1 += 1;
                    } else if self.table_scroll == max_scroll.saturating_sub(1) {
                        self.table_scroll += 1;
                        self.cursor_pos.0 = bottom_cursor_x;
                    } else {
                        self.table_scroll += 1;
                    }
                } else {
                    if self.cursor_pos.1 == max_cursor_pos_y.saturating_sub(1) {
                        self.cursor_pos.1 += max_cursor_pos_y;
                        if self.cursor_pos.0 > bottom_cursor_x {
                            self.cursor_pos.0 = bottom_cursor_x;
                        }
                    } else if self.cursor_pos.1 < max_cursor_pos_y {
                        self.cursor_pos.1 += 1;
                    } else {
                        self.cursor_pos.1 = 0;
                        self.table_scroll = 0;
                    }
                }
            }

            InputEvent::ArrowLeft => {
                if self.cursor_pos.0 > 0 {
                    self.cursor_pos.0 -= 1;
                } else if self.table_scroll == max_scroll && max_cursor_pos_y == self.cursor_pos.1 {
                    self.cursor_pos.0 = bottom_cursor_x;
                } else {
                    self.cursor_pos.0 = grid_slots.0 - 1;
                }
            }

            InputEvent::ArrowRight => {
                if self.cursor_pos.0 == bottom_cursor_x
                    && self.table_scroll == max_scroll
                    && max_cursor_pos_y == self.cursor_pos.1 {
                    self.cursor_pos.0 = 0;
                } else if self.cursor_pos.0 < grid_slots.0 - 1 {
                    self.cursor_pos.0 += 1;
                } else {
                    self.cursor_pos.0 = 0;
                }
            }
            InputEvent::Tab => todo!(),
            _ => ()
        }

        pub type AddDots = bool;
        fn shorten_str(s: &str, max_len: usize) -> (AddDots, &str) {
            // shorten with ...
            if s.len() > max_len {
                (true, &s[..max_len - 3])
            } else {
                (false, s)
            }
        }

        let max_len = self.program_state.config.gui.table.max_field_len as usize;

        {
            let (preceding, succeeding) = {
                let ind = (self.cursor_pos.1 as usize + self.table_scroll)
                    * grid_slots.0 as usize + self.cursor_pos.0 as usize;
                (ind, (items.len() - ind).saturating_sub(1))
            };

            let style = &self.program_state.config.theme.console_main.normal;
            let s = format!("{} ^ / v {}", preceding, succeeding);
            super::output_str(style, s.as_str());
            ansi::move_down(1);
            ansi::move_to_column(0);
            ansi::erase_line();
        }

        let mut first = true;
        let mut cursor_drawn = HighlightDrawn::Before;
        let mut row = 0;
        let mut col = 0;

        let mut style = &self.program_state.config.theme.console_secondary.normal;
        let upper = if self.table_scroll == max_scroll {
            if items.len() < total_slots {
                items.len()
            } else {
                total_slots - bottom_cursor_x as usize - 1
            }
        } else {
            total_slots
        };

        for (_i, item) in (0..upper)
            .map(|x| {
                let i = ((x + (self.table_scroll * grid_slots.0 as usize))
                    .rem_euclid(items.len()));
                (i, items[i])
            }) {
            if col % grid_slots.0 == 0 && !first {
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
                style = &self.program_state.config.theme.console_secondary.highlighted;
            } else if cursor_drawn == HighlightDrawn::During {
                cursor_drawn = HighlightDrawn::After;
                style = &self.program_state.config.theme.console_secondary.normal;
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

        ActionToExecute::None
    }

    fn action_on_buffer(&self, event: InputEvent) -> ActionToTake {
        match event {
            InputEvent::ArrowUp => ActionToTake::BlockBuffer,
            InputEvent::ArrowDown => ActionToTake::BlockBuffer,
            InputEvent::ArrowLeft => ActionToTake::BlockBuffer,
            InputEvent::ArrowRight => ActionToTake::BlockBuffer,
            InputEvent::Tab => ActionToTake::WriteBuffer,
            _ => ActionToTake::WriteBuffer,
        }
    }

    fn clear_output(&mut self) -> () {}
}