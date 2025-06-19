use dut::Dut;
use measurement::Measurements;
use user_text_input::TextInput;

use crate::common::*;

use super::data::suite::SuiteDataCollection;

pub mod dut;
pub mod measurement;
pub mod user_text_input;

pub struct SysContext {
    pub text_input: TextInput,
    pub measurements: Measurements,
    pub dut: Dut,
}

impl SysContext {
    pub fn new(
        suite_data: SuiteDataCollection,
        event_tx: UnboundedSender<Event>,
        action_rx: broadcast::Receiver<Action>,
    ) -> Self {
        let current_test = suite_data.current_test_ref();
        Self {
            text_input: TextInput::new(event_tx.clone(), action_rx, current_test.clone()),
            measurements: Measurements::new(current_test.clone()),
            dut: Dut::new(suite_data),
        }
    }
}
