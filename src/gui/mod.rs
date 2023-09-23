use crate::{ansi, buffer, config::theme, state};
use crate::ansi::TerminalXY;
use crate::gui::terminal::CursorPos;
use crate::input::InputEvent;

pub mod table;
pub mod dropdown;
pub mod terminal;

/// When a GUITrait is active, a keyboard event has the option of being blocked by that GUITrait.
/// If this is the case, `PropagateAction = false` which, generally, results in the `InputBuffer`
/// not being updatedgui::AdditionalView::Table(gui::table::TableGUI::init(&program_state)).
#[derive(Debug, PartialEq)]
pub enum ActionToTake {
    BlockBuffer,
    WriteBuffer(ActionType),
}

#[derive(Debug, PartialEq)]
pub enum ActionType {
    Standard,
    Other(ActionToExecute),
}

#[derive(Debug, PartialEq)]
pub enum ActionToExecute {
    SetClosestMatch(String),
}

#[derive(Debug, PartialEq)]
pub enum AdditionalViewDraw {
    BeforeBuffer,
    AfterBuffer
}

pub trait GUITrait<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self;
    fn action_before_write(
        &mut self,
        event: InputEvent,
        buffer: &buffer::InputBuffer,
        term_size: TerminalXY,
        write_from_line: u16,
        cursor_pos: CursorPos,
        arg_pos: CursorPos,
    ) -> ActionToTake;
    fn write_output(
        &mut self,
        event: InputEvent,
        term_size: TerminalXY,
        write_from_line: u16,
        buf: &buffer::InputBuffer,
    );
    fn clear_output(&mut self, write_from_line: u16) -> ();
}

pub fn output_str(style: &theme::Style, s: &str) {
    print!("{}{}", style.escape_sequence, s);
    ansi::reset();
}

#[derive(Debug, PartialEq)]
pub enum AdditionalViewNoData {
    Table,
    Dropdown,
}

pub enum AdditionalView<'a> {
    Table(table::TableGUI<'a>),
    Dropdown(dropdown::DropdownGUI<'a>),
}

impl<'a> AdditionalView<'a> {
    pub fn from_enum(view: AdditionalViewNoData, program_state: &'a state::ProgramState) -> Self {
        match view {
            AdditionalViewNoData::Table => Self::Table(table::TableGUI::init(program_state)),
            AdditionalViewNoData::Dropdown => Self::Dropdown(dropdown::DropdownGUI::init(program_state)),
        }
    }

    pub fn additional_view_no_data(&self) -> AdditionalViewNoData {
        match self {
            Self::Table(_) => AdditionalViewNoData::Table,
            Self::Dropdown(_) => AdditionalViewNoData::Dropdown,
        }
    }
}

impl<'a> GUITrait<'a> for AdditionalView<'a> {
    #[allow(unused_variables)]
    fn init(program_state: &'a state::ProgramState) -> Self {
        panic!("Cannot init AdditionalView through enum")
    }

    fn action_before_write(&mut self, event: InputEvent, buffer: &buffer::InputBuffer, term_size: TerminalXY, write_from_line: u16, cursor_pos: CursorPos, arg_pos: CursorPos) -> ActionToTake {
        match self {
            Self::Table(table) => table.action_before_write(event, buffer, term_size, write_from_line, cursor_pos, arg_pos),
            Self::Dropdown(dropdown) => dropdown.action_before_write(event, buffer, term_size, write_from_line, cursor_pos, arg_pos)
        }
    }

    fn write_output(&mut self, event: InputEvent, term_size: TerminalXY, write_from_line: u16, buf: &buffer::InputBuffer) {
        match self {
            Self::Table(table) => table.write_output(event, term_size, write_from_line, buf),
            Self::Dropdown(dropdown) => dropdown.write_output(event, term_size, write_from_line, buf, ),
        }
    }

    fn clear_output(&mut self, write_from_line: u16) -> () {
        match self {
            Self::Table(table) => table.clear_output(write_from_line),
            Self::Dropdown(dropdown) => dropdown.clear_output(write_from_line),
        }
    }
}