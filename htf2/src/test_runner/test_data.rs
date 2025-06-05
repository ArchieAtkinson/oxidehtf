use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Instant,
};

use color_eyre::Result;
use indexmap::IndexMap;
use tokio::sync::{mpsc, RwLock};

use crate::events::Event;

use super::context::measurement::MeasurementDefinition;

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
pub struct TestDataManager {
    inner: Arc<RwLock<TestData>>,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl TestDataManager {
    pub fn new(names: Vec<&'static str>, event_tx: mpsc::UnboundedSender<Event>) -> Self {
        let test_data = TestData {
            test_metadata: names
                .iter()
                .map(|n| TestMetadata {
                    name: *n,
                    state: TestState::InQueue,
                    user_data: IndexMap::new(),
                    start_time: Instant::now(),
                    end_time: Instant::now(),
                })
                .collect(),
            current_index: 0,
            dut_id: String::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(test_data)),
            event_tx,
        }
    }

    pub fn blocking_write<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut TestData) -> Result<R>,
    {
        let mut data_guard = self.inner.blocking_write();
        let result = f(&mut data_guard);
        drop(data_guard);
        self.event_tx.send(Event::UpdatedTestData)?;
        result
    }

    pub fn blocking_read<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&TestData) -> Result<R>,
    {
        let mut data_guard = self.inner.blocking_read();
        let result = f(&mut data_guard);
        drop(data_guard);
        result
    }

    pub fn blocking_get_copy(&self) -> TestData {
        self.inner.blocking_read().clone()
    }

    pub async fn get_copy(&self) -> TestData {
        self.inner.read().await.clone()
    }
}

#[derive(Debug, Clone)]
pub struct TestMetadata {
    pub name: &'static str,
    pub start_time: Instant,
    pub end_time: Instant,
    pub state: TestState,
    pub user_data: IndexMap<String, MeasurementDefinition>,
}

#[derive(Debug, Clone)]
pub struct TestData {
    pub dut_id: String,
    pub test_metadata: Vec<TestMetadata>,
    pub current_index: usize,
}

impl TestData {
    pub fn current_test(&self) -> &TestMetadata {
        let current_index = self.current_index;
        self.test_metadata
            .get(current_index)
            .expect("current_index should always be a valid index for test_metadata")
    }

    pub fn current_test_mut(&mut self) -> &mut TestMetadata {
        let current_index = self.current_index;
        self.test_metadata
            .get_mut(current_index)
            .expect("current_index should always be a valid index for test_metadata")
    }
}

impl Deref for TestData {
    type Target = Vec<TestMetadata>;

    fn deref(&self) -> &Self::Target {
        &self.test_metadata
    }
}

impl DerefMut for TestData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.test_metadata
    }
}
