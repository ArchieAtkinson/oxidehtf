use std::{cell::Cell, time::Duration};

use super::{
    blocking_read, blocking_write,
    current_test::{CurrentTestData, CurrentTestDataIterator},
    TestData, TestState,
};
use crate::common::*;
use chrono::{DateTime, FixedOffset, Utc};
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct SuiteData {
    inner: Arc<RwLock<SuiteDataRaw>>,
    event_tx: UnboundedSender<Event>,
    current_test: CurrentTestData,
}

#[derive(Debug, Clone)]
pub struct SuiteDataRaw {
    pub start_time: DateTime<FixedOffset>,
    pub dut_id: String,
    pub test_metadata: Vec<TestData>,
    pub current_index: usize,
}

impl SuiteData {
    pub fn new(names: Vec<&'static str>, event_tx: UnboundedSender<Event>) -> Self {
        let test_data = SuiteDataRaw {
            test_metadata: names
                .iter()
                .map(|n| TestData {
                    name: *n,
                    state: TestState::InQueue,
                    user_data: IndexMap::new(),
                    duration: Duration::default(),
                })
                .collect(),
            current_index: 0,
            dut_id: String::new(),
            start_time: Default::default(),
        };

        let inner = Arc::new(RwLock::new(test_data));

        Self {
            inner: inner.clone(),
            event_tx: event_tx.clone(),
            current_test: CurrentTestData::new(inner.clone(), event_tx.clone()),
        }
    }

    pub fn current_testdata_iter(&self) -> CurrentTestDataIterator {
        CurrentTestDataIterator {
            current_test: &self.current_test,
            current_index: Cell::new(0),
            number_of_tests: self.number_of_tests().unwrap(),
        }
    }

    pub fn current_test_ref(&self) -> &CurrentTestData {
        &self.current_test
    }

    pub fn current_test_copy(&self) -> CurrentTestData {
        self.current_test.clone()
    }

    pub fn set_suite_start_time(&self) -> Result<()> {
        blocking_write(&self.inner, &self.event_tx, |d| {
            let fixed_offset = FixedOffset::west_opt(0).unwrap();
            d.start_time = Utc::now().with_timezone(&fixed_offset);
            Ok(())
        })
    }

    pub fn number_of_tests(&self) -> Result<usize> {
        blocking_read(&self.inner, |d| Ok(d.test_metadata.len()))
    }

    pub fn set_dut_id(&self, id: impl Into<String>) {
        blocking_write(&self.inner, &self.event_tx, |d| {
            d.dut_id = id.into();
            Ok(())
        })
        .unwrap();
    }

    pub fn blocking_get_raw_copy(&self) -> SuiteDataRaw {
        self.inner.blocking_read().clone()
    }

    pub async fn get_raw_copy(&self) -> SuiteDataRaw {
        self.inner.read().await.clone()
    }
}

impl SuiteDataRaw {
    pub fn current_test(&self) -> &TestData {
        let current_index = self.current_index;
        self.test_metadata
            .get(current_index)
            .expect("current_index should always be a valid index for test_metadata")
    }

    pub fn current_test_mut(&mut self) -> &mut TestData {
        let current_index = self.current_index;
        self.test_metadata
            .get_mut(current_index)
            .expect("current_index should always be a valid index for test_metadata")
    }
}
