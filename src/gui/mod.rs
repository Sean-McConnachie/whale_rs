use crate::{config::theme, state};
use crate::ansi::TerminalXY;
use crate::input::InputEvent;

pub mod table;
pub mod terminal;

/// When a GUITrait is active, a keyboard event has the option of being blocked by that GUITrait.
/// If this is the case, `PropagateAction = false` which, generally, results in the `InputBuffer`
/// not being updated.
pub type PropagateAction = bool;

pub trait GUITrait<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self;
    fn write_output(&mut self, event: InputEvent, term_size: TerminalXY);
    fn action_on_buffer(&self, event: InputEvent) -> PropagateAction;
    fn clear_output(&mut self) -> ();
}

pub fn output_str(style: &theme::Style, s: &str) {
    print!("{}{}", style.escape_sequence, s);
}

pub enum AdditionalView<'a> {
    Table(table::TableGUI<'a>)
}

impl<'a> GUITrait<'a> for AdditionalView<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self {
        panic!("Don't call this through the enum!")
    }

    fn write_output(&mut self, event: InputEvent, term_size: TerminalXY) {
        match self {
            Self::Table(table) => table.write_output(event, term_size),
        };
    }

    fn action_on_buffer(&self, event: InputEvent) -> PropagateAction {
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