use crate::{input, state};

pub struct TableGUI<'a> {
    program_state: &'a state::ProgramState,
}

impl<'a> super::GUITrait<'a> for TableGUI<'a> {
    fn init(program_state: &'a state::ProgramState) -> Self {
        Self { program_state }
    }

    fn write_output(
        &mut self,
        event: input::InputEvent,
        term_size: super::TerminalXY,
    ) -> super::PropagateAction {
        unimplemented!()
    }

    fn clear_output(&mut self) -> () {}
}
