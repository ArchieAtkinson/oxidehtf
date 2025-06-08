use crossterm::event::{KeyCode, KeyModifiers};

use crate::{actions::Action, events::Event};

pub struct MovementHandler {}

impl MovementHandler {
    pub fn handle_event(event: Event) -> Option<Action> {
        let Event::Key(key) = event else {
            return None;
        };

        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Char('k')) => Some(Action::MoveUp),
            (KeyModifiers::NONE, KeyCode::Char('j')) => Some(Action::MoveDown),
            (_, _) => None,
        }
    }
}
