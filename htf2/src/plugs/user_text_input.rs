use std::sync::OnceLock;

use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

use crate::events::Event;

use super::Plug;

pub struct TextInput {
    event_tx: Option<UnboundedSender<Event>>,
}

pub static USER_INPUT_RX: OnceLock<Mutex<UnboundedReceiver<String>>> = OnceLock::new();

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
    pub fn new() -> Self {
        Self { event_tx: None }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        self.event_tx
            .as_ref()
            .expect("Event Tx Not Provided")
            .send(Event::UserInputPrompt(prompt.into()))
            .expect("Failed to send");

        USER_INPUT_RX.wait();

        USER_INPUT_RX
            .get()
            .expect("User Input TX Not present")
            .blocking_lock()
            .blocking_recv()
            .expect("No Input")
    }
}

impl Plug for TextInput {
    fn register_event_handler(&mut self, tx: UnboundedSender<Event>) {
        self.event_tx = Some(tx);
    }
}
