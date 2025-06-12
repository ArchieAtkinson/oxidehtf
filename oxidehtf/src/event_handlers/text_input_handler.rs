use crossterm::event::{KeyCode, KeyModifiers};
use tui_input::InputRequest;

use crate::common::*;

pub struct TextInputHandler {}

impl TextInputHandler {
    pub fn handle_events(event: Event) -> Option<Action> {
        use InputRequest::*;
        use KeyCode::*;

        let Event::Key(key_event) = event else {
            return None;
        };

        let request = match (key_event.code, key_event.modifiers) {
            (Backspace, KeyModifiers::NONE) => Some(DeletePrevChar),
            (Delete, KeyModifiers::NONE) => Some(DeleteNextChar),
            (Left, KeyModifiers::NONE) => Some(GoToPrevChar),
            (Right, KeyModifiers::NONE) => Some(GoToNextChar),
            (Char(c), KeyModifiers::NONE) => Some(InsertChar(c)),
            (Char(c), KeyModifiers::SHIFT) => Some(InsertChar(c)),
            (_, _) => None,
        };

        request.map(|v| Action::InputRequest(v))
    }
}
