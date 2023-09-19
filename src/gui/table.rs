use crate::{ansi, buffer, state};
use crate::ansi::TerminalXY;
use crate::gui::{ActionToExecute, ActionToTake};
use crate::input::InputEvent;

#[derive(PartialEq)]
enum CursorPosition {
    TopLeft,
    AnywhereElse,
    BottomRight,
}

#[derive(PartialEq)]
enum OverflowState {
    TopLeftAndBottomRight,
    TopLeftOnly,
    BottomRightOnly,
    None,
}

impl OverflowState {
    fn items_removed(&self) -> usize {
        match self {
            Self::TopLeftAndBottomRight => 2,
            Self::TopLeftOnly => 1,
            Self::BottomRightOnly => 1,
            Self::None => 0,
        }
    }
}

#[derive(PartialEq)]
enum TableRotation {
    FirstItems,
    ItemsEitherSide,
    LastItems,
}

pub struct TableGUI<'a> {
    program_state: &'a state::ProgramState,

    cursor_pos: TerminalXY,
    table_pos: TerminalXY,
    // Will only be using the y value for now

    prev_len: usize,

    cursor_position: CursorPosition,
    overflow_state: OverflowState,
    table_rotation: TableRotation,
}

impl<'a> TableGUI<'a> {
    const CURSOR_HOME: TerminalXY = (0, 0);
    const TABLE_HOME: TerminalXY = (0, 0);
}

impl<'a> super::GUITrait<'a> for TableGUI<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self {
        Self {
            program_state,
            cursor_pos: Self::CURSOR_HOME,
            table_pos: Self::TABLE_HOME,
            prev_len: 0,
            cursor_position: CursorPosition::TopLeft,
            overflow_state: OverflowState::None,
            table_rotation: TableRotation::FirstItems,
        }
    }

    fn write_output(
        &mut self,
        event: InputEvent,
        term_size: TerminalXY,
        write_from_line: u16,
        buf: &buffer::InputBuffer,
    ) -> ActionToExecute {
        ansi::move_to((0, write_from_line));
        ansi::erase_line();

        let curr_arg = buf.get_curr_arg();
        let arg = if buf.num_args() > 0 {
            buf.get_buffer_str(buf.arg_locs(curr_arg))
        } else {
            String::new()
        };
        let (arg_type, hint) = &buf.get_argument_hints()[curr_arg];
        let items = hint.get_selection().iter().filter(|s| s.starts_with(&arg)).collect::<Vec<_>>();
        if self.prev_len != items.len() {
            self.prev_len = items.len();
            self.cursor_pos = Self::CURSOR_HOME;
            self.table_pos = Self::TABLE_HOME;
        }

        // TODO FIXME
        let term_size = (65, 6);

        let grid_slots = {
            let columns = term_size.0 / self.program_state.config.gui.table.max_field_len;
            let rows = term_size.1 - write_from_line;
            // let cols = columns as usize;
            // let rows = (items.len() + cols - 1) / cols;
            (columns, rows)
        };

        let total_slots = (grid_slots.0 * grid_slots.1) as usize;

        let table_bottom = ((items.len() / grid_slots.0 as usize) as u16).saturating_sub(grid_slots.1);
        let top_left_overflow = self.overflow_state == OverflowState::TopLeftOnly
            || self.overflow_state == OverflowState::TopLeftAndBottomRight;
        let bottom_right_overflow = self.overflow_state == OverflowState::BottomRightOnly
            || self.overflow_state == OverflowState::TopLeftAndBottomRight;
        match event {
            InputEvent::ArrowLeft => match self.cursor_position {
                CursorPosition::TopLeft => self.cursor_pos.0 = grid_slots.0 - 1,
                CursorPosition::AnywhereElse => {
                    if self.cursor_pos.0 == 0 && self.cursor_pos.1 == grid_slots.1 - 1 && bottom_right_overflow {
                        self.cursor_pos.0 = grid_slots.0 - 2;
                    } else if self.cursor_pos.0 == 0 {
                        if self.table_pos.1 == table_bottom + 1 && self.cursor_pos.1 == grid_slots.1 - 1 {
                            self.cursor_pos.0 = if self.overflow_state != OverflowState::None {
                                (items.len() % grid_slots.0 as usize) as u16 - 1
                            } else {
                                (items.len() % grid_slots.0 as usize) as u16
                            }
                        } else {
                            self.cursor_pos.0 = grid_slots.0 - 1;
                        }
                    } else if self.cursor_pos.1 == 0 && self.cursor_pos.0 == 1 && top_left_overflow {
                        self.cursor_pos.0 = grid_slots.0 - 1;
                    } else {
                        self.cursor_pos.0 -= 1;
                    }
                }
                CursorPosition::BottomRight => self.cursor_pos.0 -= 1,
            }
            InputEvent::ArrowRight => match self.cursor_position {
                CursorPosition::TopLeft => self.cursor_pos.0 += 1,
                CursorPosition::AnywhereElse => {
                    if self.cursor_pos.1 == grid_slots.1 - 1 && bottom_right_overflow {
                        if self.cursor_pos.0 == grid_slots.0 - 2 {
                            self.cursor_pos.0 = 0;
                        } else {
                            self.cursor_pos.0 += 1;
                        }
                    } else {
                        let last_col = if self.table_pos.1 == table_bottom + 1 && self.cursor_pos.1 == grid_slots.1 - 1 {
                            if self.overflow_state != OverflowState::None {
                                (items.len() % grid_slots.0 as usize) as u16 - 1
                            } else {
                                (items.len() % grid_slots.0 as usize) as u16
                            }
                        } else {
                            grid_slots.0 - 1
                        };
                        if self.cursor_pos.0 == last_col {
                            if self.cursor_pos.1 == 0 && top_left_overflow {
                                self.cursor_pos.0 = 1;
                            } else {
                                self.cursor_pos.0 = 0;
                            }
                        } else {
                            self.cursor_pos.0 += 1;
                        }
                    }
                }
                CursorPosition::BottomRight => self.cursor_pos.0 = 0,
            }

            InputEvent::ArrowUp => {
                if self.cursor_pos.0 == 0 && self.cursor_pos.1 == 1 && top_left_overflow {
                    if self.table_pos.1 == 0 {
                        self.cursor_pos.1 = 0;
                    } else {
                        self.table_pos.1 -= 1;
                    }
                } else if self.cursor_pos.1 != 0 {
                    self.cursor_pos.1 -= 1;
                } else {
                    if self.table_pos.1 == 0 {
                        self.cursor_pos.1 = grid_slots.1 - 1;
                        self.table_pos.1 = if items.len() % grid_slots.0 as usize == 0 {
                            table_bottom
                        } else {
                            table_bottom + 1
                        };
                        if (self.cursor_pos.1 * grid_slots.1 + self.cursor_pos.0) as usize >= items.len() {
                            self.cursor_pos.0 = (items.len() % grid_slots.0 as usize) as u16;
                        }
                        if self.overflow_state != OverflowState::None {
                            if self.cursor_pos.0 == grid_slots.0 - 1 && items.len() % grid_slots.0 as usize != 0 {
                                self.cursor_pos.0 -= 1;
                            }
                        }
                    } else {
                        self.table_pos.1 -= 1;
                    }
                }
            }

            InputEvent::ArrowDown => {
            }

            _ => ()
        }

        self.cursor_position = match self.cursor_pos {
            Self::CURSOR_HOME => CursorPosition::TopLeft,
            (x, y) if x == grid_slots.0 - 1 && y == term_size.1 - 1 => CursorPosition::BottomRight,
            _ => CursorPosition::AnywhereElse,
        };


        self.table_rotation = if self.table_pos.1 > table_bottom {
            TableRotation::LastItems
        } else if self.table_pos.1 == 0 {
            TableRotation::FirstItems
        } else {
            TableRotation::ItemsEitherSide
        };


        self.overflow_state = match self.cursor_position {
            CursorPosition::TopLeft => {
                if items.len() > total_slots {
                    OverflowState::BottomRightOnly
                } else {
                    OverflowState::None
                }
            }
            CursorPosition::AnywhereElse => {
                if items.len() <= total_slots {
                    OverflowState::None
                } else if self.table_rotation == TableRotation::LastItems {
                    OverflowState::TopLeftOnly
                } else if self.table_rotation == TableRotation::ItemsEitherSide {
                    OverflowState::TopLeftAndBottomRight
                } else if self.table_rotation == TableRotation::FirstItems {
                    OverflowState::BottomRightOnly
                } else {
                    unreachable!()
                }
            }
            CursorPosition::BottomRight => {
                if items.len() > total_slots {
                    OverflowState::TopLeftOnly
                } else {
                    OverflowState::None
                }
            }
        };

        pub type AddDots = bool;
        fn shorten_str(s: &str, max_len: usize) -> (AddDots, &str) {
            // shorten with ...
            if s.len() > max_len {
                (true, &s[..max_len - 3])
            } else {
                (false, s)
            }
        }

        #[inline(always)]
        fn calc_ind(pos: TerminalXY, grid_slots: TerminalXY) -> usize {
            (pos.1 * grid_slots.0 + pos.0) as usize
        }

        let max_len = self.program_state.config.gui.table.max_field_len as usize;
        let start = calc_ind(self.table_pos, grid_slots);

        let overflow_items = self.overflow_state.items_removed();
        let items = &items[start..items.len().min(start + total_slots) - overflow_items];


        let mod_sub = if top_left_overflow {
            // TODO
            const MSG: &str = "more up";
            print!("{}", MSG);
            for _ in 0..(max_len - MSG.len()) {
                print!(" ");
            }
            1
        } else {
            0
        };

        let mut row = 0;
        let mut col = mod_sub as u16;
        for (i, item) in items.iter().enumerate() {
            if col % grid_slots.0 == 0 && i != 0 {
                ansi::move_down(1);
                ansi::move_to_column(0);
                ansi::erase_line();
                row += 1;
                col = 0;
            }

            let style = if row == self.cursor_pos.1 && col == self.cursor_pos.0 {
                &self.program_state.config.theme.console_secondary.highlighted
            } else {
                &self.program_state.config.theme.console_secondary.normal
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

        if bottom_right_overflow {
            // TODO
            const MSG: &str = "more items";
            print!("{}", MSG);
            for _ in 0..(max_len - MSG.len()) {
                print!(" ");
            }
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