use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    events::Event,
    test_data::{CurrentTestData, TestRunning, TestState},
};

pub struct TextInput {
    event_tx: UnboundedSender<Event>,
    input_rx: UnboundedReceiver<String>,
    current_test: CurrentTestData,
}

impl TextInput {
    pub fn new(
        event_tx: UnboundedSender<Event>,
        input_rx: UnboundedReceiver<String>,
        current_test: CurrentTestData,
    ) -> Self {
        Self {
            event_tx,
            input_rx,
            current_test,
        }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        let prompt = prompt.into();

        self.event_tx
            .send(Event::UserInputPrompt(prompt.into()))
            .expect("Failed to send user Prompt");

        self.current_test
            .set_state(TestState::Running(TestRunning::WaitingForInput))
            .unwrap();

        let input = self.input_rx.blocking_recv().expect("No Input");

        self.current_test
            .set_state(TestState::Running(TestRunning::Running))
            .unwrap();

        input
    }
}
