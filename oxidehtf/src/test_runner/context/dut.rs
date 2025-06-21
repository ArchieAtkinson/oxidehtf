use crate::common::*;

use super::user_text_input::TextInput;

pub struct Dut {
    action_tx: broadcast::Sender<Action>,
}

impl Dut {
    pub fn new(action_tx: broadcast::Sender<Action>) -> Self {
        Self { action_tx }
    }

    pub fn set_id(&self, id: impl Into<String>) {
        self.action_tx
            .send(Action::SetCurrentSuiteDut(id.into()))
            .expect("Action Channel closed");
    }

    pub fn set_via_operator(&self, text_input: &mut TextInput) {
        let input = text_input.request("Enter DUT:");
        self.set_id(input);
    }
}
