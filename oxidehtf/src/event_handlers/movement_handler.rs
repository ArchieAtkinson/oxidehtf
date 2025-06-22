use crossterm::event::{KeyCode, KeyModifiers};

use crate::common::*;

pub struct MovementHandler {}

impl MovementHandler {
    pub fn handle_event(event: &Event) -> Option<Action> {
        if let Event::Key(key) = event {
            return match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Char('k')) => Some(Action::MoveUp),
                (KeyModifiers::NONE, KeyCode::Char('j')) => Some(Action::MoveDown),
                (_, _) => None,
            };
        };

        None
    }
}
