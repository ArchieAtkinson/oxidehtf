use crate::test_runner::TestDataManager;

use super::user_text_input::TextInput;

pub struct DUT {
    test_data: TestDataManager,
}

impl DUT {
    pub fn new(test_data: TestDataManager) -> Self {
        Self { test_data }
    }
    pub fn set_id(&self, id: impl Into<String>) {
        self.test_data
            .blocking_write(|d| {
                d.dut_id = id.into();
                Ok(())
            })
            .unwrap();
    }
    pub fn set_via_operator(&self, text_input: &mut TextInput) {
        let input = text_input.request("Enter DUT:");

        self.set_id(input);
    }
}
