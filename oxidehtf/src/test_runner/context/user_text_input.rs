use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    events::Event,
    test_runner::test_data::{TestDataManager, TestRunning, TestState},
};

pub struct TextInput {
    event_tx: UnboundedSender<Event>,
    input_rx: UnboundedReceiver<String>,
    test_data: TestDataManager,
}

impl TextInput {
    pub fn new(
        event_tx: UnboundedSender<Event>,
        input_rx: UnboundedReceiver<String>,
        test_data: TestDataManager,
    ) -> Self {
        Self {
            event_tx,
            input_rx,
            test_data,
        }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        let prompt = prompt.into();

        self.event_tx
            .send(Event::UserInputPrompt(prompt.into()))
            .expect("Failed to send user Prompt");

        self.test_data
            .blocking_write(|d| {
                d.current_test_mut().state = TestState::Running(TestRunning::WaitingForInput);
                Ok(())
            })
            .unwrap();

        let input = self.input_rx.blocking_recv().expect("No Input");

        self.test_data
            .blocking_write(|d| {
                d.current_test_mut().state = TestState::Running(TestRunning::Running);
                Ok(())
            })
            .unwrap();

        input
    }
}
