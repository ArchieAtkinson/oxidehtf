use crossterm::event::{KeyCode, KeyModifiers};
use tui_input::InputRequest;

use crate::{actions::Action, events::Event};

pub struct TextInputHandler {}

impl TextInputHandler {
    pub fn handle_events(event: Event) -> Option<Action> {
        use InputRequest::*;
        use KeyCode::*;

        let Event::Key(key_event) = event else {
            return None;
        };

        // Taken from tui_input handle_events
        let request = match (key_event.code, key_event.modifiers) {
            (Backspace, KeyModifiers::NONE) | (Char('h'), KeyModifiers::CONTROL) => {
                Some(DeletePrevChar)
            }
            (Delete, KeyModifiers::NONE) => Some(DeleteNextChar),
            (Tab, KeyModifiers::NONE) => None,
            (Left, KeyModifiers::NONE) | (Char('b'), KeyModifiers::CONTROL) => Some(GoToPrevChar),
            (Left, KeyModifiers::CONTROL) | (Char('b'), KeyModifiers::META) => Some(GoToPrevWord),
            (Right, KeyModifiers::NONE) | (Char('f'), KeyModifiers::CONTROL) => Some(GoToNextChar),
            (Right, KeyModifiers::CONTROL) | (Char('f'), KeyModifiers::META) => Some(GoToNextWord),
            (Char('u'), KeyModifiers::CONTROL) => Some(DeleteLine),

            (Char('w'), KeyModifiers::CONTROL)
            | (Char('d'), KeyModifiers::META)
            | (Backspace, KeyModifiers::META)
            | (Backspace, KeyModifiers::ALT) => Some(DeletePrevWord),

            (Delete, KeyModifiers::CONTROL) => Some(DeleteNextWord),
            (Char('k'), KeyModifiers::CONTROL) => Some(DeleteTillEnd),
            (Char('a'), KeyModifiers::CONTROL) | (Home, KeyModifiers::NONE) => Some(GoToStart),
            (Char('e'), KeyModifiers::CONTROL) | (End, KeyModifiers::NONE) => Some(GoToEnd),
            (Char(c), KeyModifiers::NONE) => Some(InsertChar(c)),
            (Char(c), KeyModifiers::SHIFT) => Some(InsertChar(c)),
            (_, _) => None,
        };

        request.map(|v| Action::InputRequest(v))
    }
}
