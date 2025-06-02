use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::events::Event;

pub struct TextInput {
    event_tx: UnboundedSender<Event>,
    input_rx: UnboundedReceiver<String>,
}

#[derive(Debug, Clone)]
pub struct UserInput {
    pub prompt: String,
    pub input: String,
}

impl UserInput {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            input: Default::default(),
        }
    }
}

impl TextInput {
    pub fn new(event_tx: UnboundedSender<Event>, input_rx: UnboundedReceiver<String>) -> Self {
        Self { event_tx, input_rx }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        self.event_tx
            .send(Event::UserInputPrompt(prompt.into()))
            .expect("Failed to send");

        self.input_rx.blocking_recv().expect("No Input")
    }
}
