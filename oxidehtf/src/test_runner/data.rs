use std::time::Duration;

use indexmap::IndexMap;
use suite::{SuiteDataCollectionHolder, SuiteDataRaw};

pub mod current_test;
pub mod suite;

use crate::common::*;
use crate::test_runner::MeasurementDefinition;

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
pub struct TestData {
    pub name: &'static str,
    pub duration: Duration,
    pub state: TestState,
    pub user_data: IndexMap<String, MeasurementDefinition>,
}

fn blocking_write<F, R>(
    inner: &SuiteDataCollectionHolder,
    tx: &UnboundedSender<Event>,
    f: F,
) -> Result<R>
where
    F: FnOnce(&mut Vec<SuiteDataRaw>) -> Result<R>,
{
    let mut data_guard = inner.inner.blocking_write();
    let result = f(&mut data_guard);
    drop(data_guard);
    tx.send(Event::UpdatedTestData)?;
    result
}

fn blocking_read<F, R>(inner: &SuiteDataCollectionHolder, f: F) -> Result<R>
where
    F: FnOnce(&Vec<SuiteDataRaw>) -> Result<R>,
{
    let mut data_guard = inner.inner.blocking_read();
    let result = f(&mut data_guard);
    drop(data_guard);
    result
}
