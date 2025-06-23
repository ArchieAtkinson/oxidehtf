use crate::{common::*, TestFailure};

use super::user_text_input::TextInput;

pub struct Dut {
    event_tx: UnboundedSender<Event>,
}

impl Dut {
    pub fn new(event_tx: UnboundedSender<Event>) -> Self {
        Self { event_tx }
    }

    pub fn set_id(&self, id: impl Into<String>) -> Result<(), TestFailure> {
        self.event_tx
            .send(Event::CurrentSuiteDut(id.into()))
            .or(Err(TestFailure::SystemExited))
    }

    pub fn set_via_operator(&self, text_input: &mut TextInput) -> Result<(), TestFailure> {
        let input = text_input.request("Enter DUT:")?;
        self.set_id(input)
    }
}
