use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::events::Event;

use super::{TestData, TestState};

pub struct TextInput {
    data: Arc<RwLock<TestData>>,
    event_tx: mpsc::UnboundedSender<Event>,
    input_tx: mpsc::UnboundedReceiver<String>,
}

impl TextInput {
    pub fn new(
        data: Arc<RwLock<TestData>>,
        event_tx: mpsc::UnboundedSender<Event>,
        input_tx: mpsc::UnboundedReceiver<String>,
    ) -> Self {
        Self {
            data,
            event_tx,
            input_tx,
        }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        {
            let mut locked_data = self.data.blocking_write();
            let current_index = locked_data.current_index;
            locked_data[current_index]
                .user_inputs
                .push(super::UserInput::new(prompt));
            locked_data[current_index].state =
                TestState::Running(super::TestRunning::WaitingForInput);

            self.event_tx.send(Event::UpdatedTestData).expect("Oops");
        }

        let input = self.input_tx.blocking_recv().expect("Failed to get input");

        let mut locked_data = self.data.blocking_write();
        let current_index = locked_data.current_index;
        locked_data[current_index]
            .user_inputs
            .last_mut()
            .expect("Should be populated")
            .input = input.clone();
        locked_data[current_index].state = TestState::Running(super::TestRunning::Running);

        input
    }
}
