use std::sync::Arc;

use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    RwLock,
};

use crate::{
    events::Event,
    test_runner::{TestData, TestRunning, TestState},
};

pub struct TextInput {
    event_tx: UnboundedSender<Event>,
    input_rx: UnboundedReceiver<String>,
    test_data: Arc<RwLock<TestData>>,
}

impl TextInput {
    pub fn new(
        event_tx: UnboundedSender<Event>,
        input_rx: UnboundedReceiver<String>,
        test_data: Arc<RwLock<TestData>>,
    ) -> Self {
        Self {
            event_tx,
            input_rx,
            test_data,
        }
    }

    pub fn request(&mut self, prompt: impl Into<String>) -> String {
        let prompt = prompt.into();
        self.test_data
            .blocking_write()
            .current_test()
            .user_inputs
            .insert(prompt.clone(), String::new());

        self.test_data.blocking_write().current_test().state =
            TestState::Running(TestRunning::WaitingForInput);

        self.event_tx
            .send(Event::UpdatedTestData)
            .expect("Failed to send event");

        let input = self.input_rx.blocking_recv().expect("No Input");

        let lock = &mut self.test_data.blocking_write();
        *lock
            .current_test()
            .user_inputs
            .get_mut(&prompt)
            .expect("No Inputs Requested") = input.clone();

        lock.current_test().state = TestState::Running(TestRunning::Running);

        self.event_tx
            .send(Event::UpdatedTestData)
            .expect("Failed to send event");

        input
    }
}
