use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::events::Event;

use super::TestRunnerState;

pub struct TextInput {
    state: Arc<RwLock<TestRunnerState>>,
    event_tx: mpsc::UnboundedSender<Event>,
    input_alert: mpsc::UnboundedReceiver<()>,
}

impl TextInput {
    pub fn new(
        state: Arc<RwLock<TestRunnerState>>,
        event_tx: mpsc::UnboundedSender<Event>,
        input_alert: mpsc::UnboundedReceiver<()>,
    ) -> Self {
        Self {
            state,
            event_tx,
            input_alert,
        }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        {
            let mut lock = self.state.blocking_write();
            let current_index = lock.current_index;
            lock.tests[current_index]
                .data
                .user_input
                .push((prompt.into(), "".into()));

            self.event_tx
                .send(Event::UpdatedTestRunnerState)
                .expect("Oops");
        }

        self.input_alert.blocking_recv();

        let lock = self.state.blocking_read();
        let current_index = lock.current_index;
        lock.tests[current_index]
            .data
            .user_input
            .last()
            .expect("Should be populated")
            .1
            .clone()
    }
}
