use crate::{ansi, buffer, config::theme, state};
use crate::ansi::TerminalXY;
use crate::input::InputEvent;

pub mod table;
pub mod terminal;

/// When a GUITrait is active, a keyboard event has the option of being blocked by that GUITrait.
/// If this is the case, `PropagateAction = false` which, generally, results in the `InputBuffer`
/// not being updatedgui::AdditionalView::Table(gui::table::TableGUI::init(&program_state)).
#[derive(Debug, PartialEq)]
pub enum ActionToTake {
    BlockBuffer,
    WriteBuffer,
}

#[derive(Debug, PartialEq)]
pub enum ActionToExecute {
    None,
    SetClosestMatch(String),
}

pub trait GUITrait<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self;
    fn write_output(&mut self, event: InputEvent, term_size: TerminalXY, write_from_line: u16, buf: &buffer::InputBuffer) -> ActionToExecute;
    fn action_on_buffer(&self, event: InputEvent) -> ActionToTake;
    fn clear_output(&mut self) -> ();
}

pub fn output_str(style: &theme::Style, s: &str) {
    print!("{}{}", style.escape_sequence, s);
    ansi::reset();
}

#[derive(Debug, PartialEq)]
pub enum AdditionalViewNoData {
    Table,
}

pub enum AdditionalView<'a> {
    Table(table::TableGUI<'a>)
}

impl<'a> AdditionalView<'a> {
    pub fn from_enum(view: AdditionalViewNoData, program_state: &'a state::ProgramState) -> Self {
        match view {
            AdditionalViewNoData::Table => Self::Table(table::TableGUI::init(program_state)),
        }
    }

    pub fn additional_view_no_data(&self) -> AdditionalViewNoData {
        match self {
            Self::Table(_) => AdditionalViewNoData::Table,
        }
    }
}

impl<'a> GUITrait<'a> for AdditionalView<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self {
        panic!("Cannot init AdditionalView through enum")
    }

    fn write_output(&mut self, event: InputEvent, term_size: TerminalXY, write_from_line: u16, buf: &buffer::InputBuffer) -> ActionToExecute {
        match self {
            Self::Table(table) => table.write_output(event, term_size, write_from_line, buf),
        }
    }

    fn action_on_buffer(&self, event: InputEvent) -> ActionToTake {
        match self {
            Self::Table(table) => table.action_on_buffer(event)
        }
    }

    fn clear_output(&mut self) -> () {
        match self {
            Self::Table(table) => table.clear_output()
        }
    }
}