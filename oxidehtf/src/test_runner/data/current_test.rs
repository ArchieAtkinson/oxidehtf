use std::cell::Cell;
use std::time::Duration;

use super::{blocking_write, suite::SuiteDataRaw, TestState};
use crate::{common::*, test_runner::MeasurementDefinition};

#[derive(Debug, Clone)]
pub struct CurrentTestData {
    inner: Arc<RwLock<SuiteDataRaw>>,
    event_tx: UnboundedSender<Event>,
}

impl CurrentTestData {
    pub fn new(inner: Arc<RwLock<SuiteDataRaw>>, event_tx: UnboundedSender<Event>) -> Self {
        Self { inner, event_tx }
    }

    pub fn update_test_index(&self, index: usize) -> Result<()> {
        blocking_write(&self.inner, &self.event_tx, |d| {
            d.current_index = index;
            Ok(())
        })
    }

    pub fn set_state(&self, state: TestState) -> Result<()> {
        blocking_write(&self.inner, &self.event_tx, |d| {
            d.current_test_mut().state = state;
            Ok(())
        })
    }

    pub fn set_test_duration(&self, duration: Duration) -> Result<()> {
        blocking_write(&self.inner, &self.event_tx, |d| {
            d.current_test_mut().duration = duration;
            Ok(())
        })
    }

    pub fn insert_measurement(&self, name: &str, def: MeasurementDefinition) -> Result<()> {
        blocking_write(&self.inner, &self.event_tx, |d| {
            d.current_test_mut()
                .user_data
                .insert(name.into(), def.clone());
            Ok(())
        })
    }
}

pub struct CurrentTestDataIterator<'a> {
    pub current_test: &'a CurrentTestData,
    pub current_index: Cell<usize>,
    pub number_of_tests: usize,
}

impl<'a> Iterator for &'a CurrentTestDataIterator<'a> {
    type Item = &'a CurrentTestData;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current_index.get();

        if index >= self.number_of_tests {
            return None;
        }
        self.current_test.update_test_index(index).ok()?;
        self.current_index.replace(index + 1);
        Some(self.current_test)
    }
}
