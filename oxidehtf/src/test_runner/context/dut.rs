use crate::common::*;

use super::user_text_input::TextInput;

pub struct Dut {
    event_tx: UnboundedSender<Event>,
}

impl Dut {
    pub fn new(event_tx: UnboundedSender<Event>) -> Self {
        Self { event_tx }
    }

    pub fn set_id(&self, id: impl Into<String>) {
        self.event_tx
            .send(Event::CurrentSuiteDut(id.into()))
            .expect("Action Channel closed");
    }

    pub fn set_via_operator(&self, text_input: &mut TextInput) {
        let input = text_input.request("Enter DUT:");
        self.set_id(input);
    }
}
