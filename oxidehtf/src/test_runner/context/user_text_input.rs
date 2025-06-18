use crate::{
    common::*,
    test_runner::{CurrentTestData, TestRunning, TestState},
};

pub struct TextInput {
    event_tx: UnboundedSender<Event>,
    action_rx: broadcast::Receiver<Action>,
    current_test: CurrentTestData,
}

impl TextInput {
    pub fn new(
        event_tx: UnboundedSender<Event>,
        action_rx: broadcast::Receiver<Action>,
        current_test: CurrentTestData,
    ) -> Self {
        Self {
            event_tx,
            action_rx,
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

        let input = loop {
            if let Ok(action) = self.action_rx.blocking_recv() {
                match action {
                    Action::UserInputValue(s) => break s,
                    _ => (),
                }
            } else {
                panic!("Channel Closed");
            }
        };

        self.current_test
            .set_state(TestState::Running(TestRunning::Running))
            .unwrap();

        input
    }
}
