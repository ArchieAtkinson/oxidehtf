use std::time::Duration;

use indexmap::IndexMap;

pub mod suite;

// use crate::common::*;
use crate::test_runner::MeasurementDefinition;

use super::TestFailure;

#[derive(Clone, Default, Debug, PartialEq)]
pub enum TestState {
    #[default]
    InQueue,
    Running(TestRunning),
    Done(TestDone),
}

#[derive(Clone, Default, Debug, PartialEq)]
pub enum TestRunning {
    #[default]
    Running,
    WaitingForInput,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub enum TestDone {
    #[default]
    Passed,
    Failed(TestFailure),
}

#[derive(Debug, Clone)]
pub struct TestData {
    pub name: &'static str,
    pub duration: Duration,
    pub state: TestState,
    pub user_data: IndexMap<String, MeasurementDefinition>,
}
