use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    events::Event,
    test_runner::test_data::{blocking_write, SuiteData, TestRunning, TestState},
};

pub struct TextInput {
    event_tx: UnboundedSender<Event>,
    input_rx: UnboundedReceiver<String>,
    test_data: SuiteData,
}

impl TextInput {
    pub fn new(
        event_tx: UnboundedSender<Event>,
        input_rx: UnboundedReceiver<String>,
        test_data: SuiteData,
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

        blocking_write(&self.test_data.inner, &self.test_data.event_tx, |d| {
            d.current_test_mut().state = TestState::Running(TestRunning::WaitingForInput);
            Ok(())
        })
        .unwrap();

        let input = self.input_rx.blocking_recv().expect("No Input");

        blocking_write(&self.test_data.inner, &self.test_data.event_tx, |d| {
            d.current_test_mut().state = TestState::Running(TestRunning::Running);
            Ok(())
        })
        .unwrap();

        input
    }
}
