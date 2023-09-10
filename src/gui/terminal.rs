use crate::{state, utils};

pub struct TerminalGUI<'a, T>
where
    T: super::GUITrait<'a>,
{
    program_state: &'a state::ProgramState,

    additional_view: Option<T>,
}

impl<'a, T> TerminalGUI<'a, T>
where T: super::GUITrait<'a>
{
    // super::GUITrait<'a> for
    fn init(program_state: &'a state::ProgramState) -> Self {
        Self { program_state, additional_view: None }
    }

    fn write_output(term_size: super::TerminalXY) -> () {}

    fn clear_output() -> () {}
}
