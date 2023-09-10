use crate::{config::theme, input, state};

pub mod table;
pub mod terminal;

pub type TerminalXY = (usize, usize);

/// When a GUITrait is active, a keyboard event has the option of being blocked by that GUITrait.
/// If this is the case, `PropagateAction = false` which, generally, results in the `InputBuffer`
/// not being updated.
pub type PropagateAction = bool;

pub trait GUITrait<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self;
    fn write_output(&mut self, event: input::InputEvent, term_size: TerminalXY) -> PropagateAction;
    fn clear_output(&mut self) -> ();
}

pub fn output_str(style: &theme::Style, s: &str) {
    print!("{}{}", style.escape_sequence, s);
}
