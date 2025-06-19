use crate::test_runner::data::suite::SuiteDataCollection;

use super::user_text_input::TextInput;

pub struct Dut {
    suite_data: SuiteDataCollection,
}

impl Dut {
    pub fn new(test_data: SuiteDataCollection) -> Self {
        Self {
            suite_data: test_data,
        }
    }

    pub fn set_id(&self, id: impl Into<String>) {
        self.suite_data.set_dut_id(id);
    }

    pub fn set_via_operator(&self, text_input: &mut TextInput) {
        let input = text_input.request("Enter DUT:");
        self.set_id(input);
    }
}
