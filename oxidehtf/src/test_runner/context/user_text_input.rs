use tokio::sync::oneshot;

use crate::{
    common::*,
    test_runner::{data::suite::SuiteDataCollection, TestRunning, TestState},
};

pub struct TextInput {
    event_tx: UnboundedSender<Event>,
    suites_data: SuiteDataCollection,
}

impl TextInput {
    pub fn new(event_tx: UnboundedSender<Event>, suites_data: SuiteDataCollection) -> Self {
        Self {
            event_tx,
            suites_data,
        }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        let prompt = prompt.into();

        let (input_tx, input_rx) = oneshot::channel::<String>();

        self.event_tx
            .send(Event::UserInputPrompt(prompt.into(), Some(input_tx)))
            .expect("Failed to send user Prompt");

        self.suites_data
            .blocking_write(|f| {
                f.current_suite_mut().current_test_mut().state =
                    TestState::Running(TestRunning::WaitingForInput);
                Ok(())
            })
            .expect("Failed to Write");

        info!("Waiting for ...");
        let input = input_rx.blocking_recv().unwrap();

        self.suites_data
            .blocking_write(|f| {
                f.current_suite_mut().current_test_mut().state =
                    TestState::Running(TestRunning::Running);
                Ok(())
            })
            .expect("Failed to write");

        input
    }
}
