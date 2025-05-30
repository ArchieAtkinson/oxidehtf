use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::events::Event;

use super::{TestRunnerState, TestState};

pub struct TextInput {
    state: Arc<RwLock<TestRunnerState>>,
    event_tx: mpsc::UnboundedSender<Event>,
    input_tx: mpsc::UnboundedReceiver<String>,
}

impl TextInput {
    pub fn new(
        state: Arc<RwLock<TestRunnerState>>,
        event_tx: mpsc::UnboundedSender<Event>,
        input_tx: mpsc::UnboundedReceiver<String>,
    ) -> Self {
        Self {
            state,
            event_tx,
            input_tx,
        }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        {
            let mut lock = self.state.blocking_write();
            let current_index = lock.current_index;
            lock.tests[current_index]
                .data
                .user_inputs
                .push(super::UserInput::new(prompt));
            lock.tests[current_index].data.state =
                TestState::Running(super::TestRunning::WaitingForInput);

            self.event_tx
                .send(Event::UpdatedTestRunnerState)
                .expect("Oops");
        }

        let input = self.input_tx.blocking_recv().expect("Failed to get input");

        let mut lock = self.state.blocking_write();
        let current_index = lock.current_index;
        lock.tests[current_index]
            .data
            .user_inputs
            .last_mut()
            .expect("Should be populated")
            .input = input.clone();
        lock.tests[current_index].data.state = TestState::Running(super::TestRunning::Running);

        input
    }
}
