use std::sync::OnceLock;

use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use crate::PlugEvent;

use super::{Plug, PlugEventSender};

pub struct TextInput {
    sender: Option<PlugEventSender>,
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
        Self { sender: None }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        self.sender
            .as_ref()
            .expect("Event Tx Not Provided")
            .send(PlugEvent::UserInputPrompt(prompt.into()))
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
    fn request_sender(&mut self, sender: PlugEventSender) {
        self.sender = Some(sender);
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self { sender: None }
    }
}
