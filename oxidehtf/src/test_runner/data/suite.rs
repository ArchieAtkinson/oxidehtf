use std::time::Duration;

use super::{TestData, TestState};
use crate::{common::*, test_runner::MeasurementDefinition};
use chrono::{DateTime, FixedOffset, Utc};
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct SuiteDataCollectionRaw {
    pub inner: Vec<SuiteData>,
    pub dut_id: String,
    pub current: usize,
}

#[derive(Debug, Clone)]
pub struct SuiteDataCollection {
    pub data: Arc<RwLock<SuiteDataCollectionRaw>>,
    pub event_tx: UnboundedSender<Event>,
}

#[derive(Debug, Clone)]
pub struct SuiteData {
    pub name: &'static str,
    pub start_time: DateTime<FixedOffset>,
    pub test_data: Vec<TestData>,
    pub current_index: usize,
}

impl SuiteDataCollection {
    pub fn new(suites_data: Vec<SuiteData>, event_tx: UnboundedSender<Event>) -> Self {
        let collection_holder = Arc::new(RwLock::new(SuiteDataCollectionRaw {
            inner: suites_data,
            dut_id: String::new(),
            current: 0,
        }));

        Self {
            data: collection_holder.clone(),
            event_tx: event_tx.clone(),
        }
    }

    pub fn blocking_write<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut SuiteDataCollectionRaw) -> Result<R>,
    {
        let mut data_guard = self.data.blocking_write();
        let result = f(&mut data_guard);
        drop(data_guard);
        self.event_tx.send(Event::UpdatedTestData)?;
        result
    }

    pub async fn write<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut SuiteDataCollectionRaw) -> Result<R>,
    {
        let mut data_guard = self.data.write().await;
        let result = f(&mut data_guard);
        drop(data_guard);
        self.event_tx.send(Event::UpdatedTestData)?;
        result
    }

    pub fn blocking_read<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&SuiteDataCollectionRaw) -> Result<R>,
    {
        let mut data_guard = self.data.blocking_read();
        let result = f(&mut data_guard);
        drop(data_guard);
        result
    }

    pub fn set_dut_id(&self, id: impl Into<String>) {
        self.blocking_write(|d| {
            d.dut_id = id.into();
            Ok(())
        })
        .unwrap();
    }

    pub fn blocking_get_raw_copy(&self) -> SuiteDataCollectionRaw {
        self.blocking_read(|d| Ok(d.clone())).unwrap()
    }

    pub async fn get_raw_copy(&self) -> SuiteDataCollectionRaw {
        self.data.read().await.clone()
    }
}

impl SuiteDataCollectionRaw {
    pub fn set_suite_start_time(&mut self) -> Result<()> {
        let fixed_offset = FixedOffset::west_opt(0).unwrap();
        self.inner[self.current].start_time = Utc::now().with_timezone(&fixed_offset);
        Ok(())
    }

    pub fn get_current_suite_index(&self) -> usize {
        self.current
    }

    pub fn current_suite(&self) -> &SuiteData {
        let current_index = self.current;
        self.inner
            .get(current_index)
            .expect("current_index should always be a valid index for test_metadata")
    }

    pub fn current_suite_mut(&mut self) -> &mut SuiteData {
        let current_index = self.current;
        self.inner
            .get_mut(current_index)
            .expect("current_index should always be a valid index for test_metadata")
    }
}

impl SuiteData {
    pub fn new(func_names: Vec<&'static str>, suite_name: &'static str) -> Self {
        Self {
            name: suite_name,
            test_data: func_names
                .iter()
                .map(|n| TestData {
                    name: *n,
                    state: TestState::InQueue,
                    user_data: IndexMap::new(),
                    duration: Duration::default(),
                })
                .collect(),
            current_index: 0,
            start_time: Default::default(),
        }
    }

    pub fn get_test_amount(&self) -> usize {
        self.test_data.len()
    }

    pub fn current_test(&self) -> &TestData {
        let current_index = self.current_index;
        self.test_data
            .get(current_index)
            .expect("current_index should always be a valid index for test_metadata")
    }

    pub fn current_test_mut(&mut self) -> &mut TestData {
        let current_index = self.current_index;
        self.test_data
            .get_mut(current_index)
            .expect("current_index should always be a valid index for test_metadata")
    }

    pub fn update_test_index(&mut self, index: usize) {
        self.current_index = index;
    }

    pub fn set_current_test_state(&mut self, state: TestState) {
        self.current_test_mut().state = state;
    }

    pub fn set_test_duration(&mut self, duration: Duration) {
        self.current_test_mut().duration = duration;
    }

    pub fn get_test_name(&self) -> &'static str {
        self.current_test().name
    }

    pub fn insert_measurement(&mut self, name: &str, def: MeasurementDefinition) {
        self.current_test_mut().user_data.insert(name.into(), def);
    }
}
