use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Esc,
    Tab,
    Enter,
    Backspace,
    Delete,
    Character(char),
    CtrlBackspace,
    CtrlDelete,

    CtrlC,
    CtrlT,
    CtrlS,
    CtrlD,

    ArrowUp,
    ArrowRight,
    ArrowDown,
    ArrowLeft,

    CtrlShiftArrowLeft,
    CtrlShiftArrowRight,

    ShiftArrowUp,
    ShiftArrowRight,
    ShiftArrowDown,
    ShiftArrowLeft,

    AltArrowLeft,
    AltArrowRight,

    CtrlArrowRight,
    CtrlArrowLeft,

    Resize((u16, u16)),

    Other(Event),
}

pub fn get_input() -> Result<InputEvent, std::io::Error> {
    let key = crossterm::event::read()?;

    Ok(match key {
        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::Esc,

        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::CtrlC,
        Event::Key(KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::CtrlT,
        Event::Key(KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::CtrlS,
        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::CtrlD,

        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::Character(c),
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::Character(c),

        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::Tab,
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::Enter,
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::Backspace,
        Event::Key(KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::Delete,

        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::CtrlBackspace,
        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => InputEvent::CtrlDelete,
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
            ..
        }) => InputEvent::CtrlBackspace,
        Event::Key(KeyEvent {
            code: KeyCode::Delete,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
            ..
        }) => InputEvent::CtrlDelete,
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
            ..
        }) => {
            if let Event::Key(key_event) = key {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    InputEvent::ShiftArrowLeft
                } else if key_event.modifiers == KeyModifiers::CONTROL {
                    InputEvent::CtrlArrowLeft
                } else if key_event.modifiers == KeyModifiers::ALT {
                    InputEvent::AltArrowLeft
                } else if key_event.modifiers == KeyModifiers::NONE {
                    InputEvent::ArrowLeft
                } else {
                    if key_event.modifiers == KeyModifiers::SHIFT | KeyModifiers::CONTROL {
                        InputEvent::CtrlShiftArrowLeft
                    } else {
                        InputEvent::Other(key)
                    }
                }
            } else {
                InputEvent::Other(key)
            }
        }

        Event::Key(KeyEvent {
            code: KeyCode::Right,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
            ..
        }) => {
            if let Event::Key(key_event) = key {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    InputEvent::ShiftArrowRight
                } else if key_event.modifiers == KeyModifiers::CONTROL {
                    InputEvent::CtrlArrowRight
                } else if key_event.modifiers == KeyModifiers::ALT {
                    InputEvent::AltArrowRight
                } else if key_event.modifiers == KeyModifiers::NONE {
                    InputEvent::ArrowRight
                } else {
                    if key_event.modifiers == KeyModifiers::SHIFT | KeyModifiers::CONTROL {
                        InputEvent::CtrlShiftArrowRight
                    } else {
                        InputEvent::Other(key)
                    }
                }
            } else {
                InputEvent::Other(key)
            }
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
            ..
        }) => {
            if let Event::Key(key_event) = key {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    InputEvent::ShiftArrowUp
                } else if key_event.modifiers == KeyModifiers::NONE {
                    InputEvent::ArrowUp
                } else {
                    InputEvent::Other(key)
                }
            } else {
                InputEvent::Other(key)
            }
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
            ..
        }) => {
            if let Event::Key(key_event) = key {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    InputEvent::ShiftArrowDown
                } else if key_event.modifiers == KeyModifiers::NONE {
                    InputEvent::ArrowDown
                } else {
                    InputEvent::Other(key)
                }
            } else {
                InputEvent::Other(key)
            }
        }

        Event::Resize(x, y) => InputEvent::Resize((x, y)),

        _ => InputEvent::Other(key),
    })
}
