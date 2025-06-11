use crate::test_runner::{test_data::blocking_write, SuiteData};

use super::user_text_input::TextInput;

pub struct DUT {
    test_data: SuiteData,
}

impl DUT {
    pub fn new(test_data: SuiteData) -> Self {
        Self { test_data }
    }
    pub fn set_id(&self, id: impl Into<String>) {
        blocking_write(&self.test_data.inner, &self.test_data.event_tx, |d| {
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
