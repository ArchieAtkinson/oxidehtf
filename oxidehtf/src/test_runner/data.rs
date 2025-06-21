use std::time::Duration;

use indexmap::IndexMap;

pub mod suite;

// use crate::common::*;
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
