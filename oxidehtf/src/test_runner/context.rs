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
    pub fn new(suite_data: SuiteDataCollection, event_tx: UnboundedSender<Event>) -> Self {
        Self {
            text_input: TextInput::new(event_tx.clone(), suite_data.clone()),
            measurements: Measurements::new(suite_data.clone()),
            dut: Dut::new(event_tx.clone()),
        }
    }
}
