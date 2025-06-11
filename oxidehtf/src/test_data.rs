use std::{cell::Cell, sync::Arc, time::Duration};

use chrono::{DateTime, FixedOffset, Utc};
use color_eyre::Result;
use indexmap::IndexMap;
use tokio::sync::{mpsc, RwLock};

use crate::events::Event;
use crate::test_runner::context::measurement::MeasurementDefinition;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum TestState {
    #[default]
    InQueue,
    Running(TestRunning),
    Done(TestDone),
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum TestRunning {
    #[default]
    Running,
    WaitingForInput,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum TestDone {
    #[default]
    Passed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SuiteData {
    inner: Arc<RwLock<SuiteDataRaw>>,
    event_tx: mpsc::UnboundedSender<Event>,
    current_test: CurrentTestData,
}

#[derive(Debug, Clone)]
pub struct SuiteDataRaw {
    pub start_time: DateTime<FixedOffset>,
    pub dut_id: String,
    pub test_metadata: Vec<TestData>,
    pub current_index: usize,
}

#[derive(Debug, Clone)]
pub struct TestData {
    pub name: &'static str,
    pub duration: Duration,
    pub state: TestState,
    pub user_data: IndexMap<String, MeasurementDefinition>,
}

#[derive(Debug, Clone)]
pub struct CurrentTestData {
    inner: Arc<RwLock<SuiteDataRaw>>,
    event_tx: mpsc::UnboundedSender<Event>,
}

fn blocking_write<F, R>(
    inner: &Arc<RwLock<SuiteDataRaw>>,
    tx: &mpsc::UnboundedSender<Event>,
    f: F,
) -> Result<R>
where
    F: FnOnce(&mut SuiteDataRaw) -> Result<R>,
{
    let mut data_guard = inner.blocking_write();
    let result = f(&mut data_guard);
    drop(data_guard);
    tx.send(Event::UpdatedTestData)?;
    result
}

fn blocking_read<F, R>(inner: &Arc<RwLock<SuiteDataRaw>>, f: F) -> Result<R>
where
    F: FnOnce(&SuiteDataRaw) -> Result<R>,
{
    let mut data_guard = inner.blocking_read();
    let result = f(&mut data_guard);
    drop(data_guard);
    result
}

impl CurrentTestData {
    fn new(inner: Arc<RwLock<SuiteDataRaw>>, event_tx: mpsc::UnboundedSender<Event>) -> Self {
        Self { inner, event_tx }
    }

    fn update_test_index(&self, index: usize) -> Result<()> {
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
    current_test: &'a CurrentTestData,
    current_index: Cell<usize>,
    number_of_tests: usize,
}

impl<'a> Iterator for &'a CurrentTestDataIterator<'a> {
    type Item = &'a CurrentTestData;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current_index.get();

        if index > self.number_of_tests {
            return None;
        }
        self.current_test.update_test_index(index).ok()?;
        self.current_index.replace(index + 1);
        Some(self.current_test)
    }
}

impl SuiteData {
    pub fn new(names: Vec<&'static str>, event_tx: mpsc::UnboundedSender<Event>) -> Self {
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
