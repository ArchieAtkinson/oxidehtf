use crate::{
    common::*,
    test_runner::{data::suite::SuiteDataCollection, TestRunning, TestState},
};

pub struct TextInput {
    event_tx: UnboundedSender<Event>,
    action_rx: broadcast::Receiver<Action>,
    suites_data: SuiteDataCollection,
}

impl TextInput {
    pub fn new(
        event_tx: UnboundedSender<Event>,
        action_rx: broadcast::Receiver<Action>,
        suites_data: SuiteDataCollection,
    ) -> Self {
        Self {
            event_tx,
            action_rx,
            suites_data,
        }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        let prompt = prompt.into();

        self.event_tx
            .send(Event::UserInputPrompt(prompt.into()))
            .expect("Failed to send user Prompt");

        self.suites_data
            .blocking_write(|f| {
                f.current_suite_mut().current_test_mut().state =
                    TestState::Running(TestRunning::WaitingForInput);
                Ok(())
            })
            .expect("Failed to Write");

        let input = loop {
            use broadcast::error::RecvError::*;
            match self.action_rx.blocking_recv() {
                Ok(action) => match action {
                    Action::UserInputValue(s) => break s,
                    _ => (),
                },
                Err(e) => match e {
                    Lagged(_) => (),
                    Closed => panic!("Channel Closed"),
                },
            }
        };

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
