use crate::{ansi, buffer, config::theme};
use crate::ansi::TerminalXY;
use crate::gui::terminal::CursorPos;
use crate::input::InputEvent;

pub mod table;
pub mod dropdown;
pub mod terminal;
pub mod explorer;

#[derive(PartialEq)]
enum HighlightDrawn {
    Before,
    During,
    After,
}

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
    SetBuffer(String),
    SetClosestMatch(String),
}

#[derive(Debug, PartialEq)]
pub enum AdditionalViewDraw {
    BeforeBuffer,
    AfterBuffer,
}

#[derive(Debug, PartialEq)]
pub enum ViewType {
    Table,
    Dropdown,
    Explorer,
}

pub enum ViewTypeData {
    None,
    Table(table::TableGUI),
    Dropdown(dropdown::DropdownGUI),
    Explorer(explorer::FileExplorerGUI),
}

pub trait GUITrait {
    fn view_type(&self) -> ViewType;
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
