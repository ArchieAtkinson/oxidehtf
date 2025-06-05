use std::sync::Arc;

use tokio::sync::RwLock;

use crate::test_runner::TestData;

use super::user_text_input::TextInput;

pub struct DUT {
    test_data: Arc<RwLock<TestData>>,
}

impl DUT {
    pub fn new(test_data: Arc<RwLock<TestData>>) -> Self {
        Self { test_data }
    }
    pub fn set_id(&self, id: impl Into<String>) {
        self.test_data.blocking_write().dut_id = id.into();
    }
    pub fn set_via_operator(&self, text_input: &mut TextInput) {
        let input = text_input.request("Enter DUT:");

        self.set_id(input);
    }
}
