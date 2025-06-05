use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Instant,
};

use cli_log::*;
use color_eyre::Result;
use context::{measurement::MeasurementDefinition, SysContext};
use errors::TestFailure;
use indexmap::IndexMap;
use lifecycle::TestLifecycle;
use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite};
use tokio::sync::{mpsc, RwLock};

use crate::events::Event;

pub mod context;
pub mod errors;
pub mod lifecycle;

pub type FuncType<T> = fn(&mut SysContext, &mut T) -> Result<(), TestFailure>;

#[derive(Debug, Clone)]
pub struct TestData {
    pub dut_id: String,
    pub data: Vec<TestMetadata>,
    pub current_index: usize,
}

impl TestData {
    pub fn current_test(&mut self) -> &mut TestMetadata {
        &mut self.data[self.current_index]
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
pub struct TestFunctions<T> {
    pub funcs: Vec<FuncType<T>>,
}

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

pub struct TestRunner<T: TestLifecycle> {
    data: Arc<RwLock<TestData>>,
    funcs: TestFunctions<T>,
    event_tx: mpsc::UnboundedSender<Event>,
    context: SysContext,
    fixture: T,
}

impl<T: TestLifecycle> TestRunner<T> {
    pub fn new(
        funcs: TestFunctions<T>,
        data: Arc<RwLock<TestData>>,
        event_tx: mpsc::UnboundedSender<Event>,
        context: SysContext,
        fixture: T,
    ) -> Self {
        Self {
            data,
            funcs,
            event_tx,
            context,
            fixture,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting Test Runner");
        let num_tests = self.data.blocking_read().data.len();

        info!("Loop");

        self.fixture.setup()?;

        for index in 0..num_tests {
            let mut data_guard = self.data.blocking_write();
            data_guard.current_index = index;
            data_guard.current_test().state = TestState::Running(TestRunning::Running);
            data_guard.current_test().start_time = Instant::now();
            drop(data_guard);
            self.event_tx.send(Event::UpdatedTestData)?;

            self.fixture.before_test()?;
            let result = (self.funcs.funcs[index])(&mut self.context, &mut self.fixture);
            self.fixture.after_test()?;

            let mut data_guard = self.data.blocking_write();
            data_guard[index].state = match result {
                Ok(_) => TestState::Done(TestDone::Passed),
                Err(e) => {
                    error!("{:#?}", e);
                    TestState::Done(TestDone::Failed)
                }
            };
            data_guard.current_test().end_time = Instant::now();
            drop(data_guard);
            self.event_tx.send(Event::UpdatedTestData)?;
        }

        self.event_tx.send(Event::TestsCompleted)?;

        self.fixture.teardown()?;

        info!("Done");

        self.produce_junit_report()?;

        Ok(())
    }

    fn produce_junit_report(&self) -> Result<()> {
        let mut report = Report::new("htf2-run");
        let mut test_suite = TestSuite::new("htf2-suite");

        let tests = self.data.blocking_read().data.clone();

        for test in tests {
            let test_case_result = match test.state {
                TestState::Done(r) => match r {
                    TestDone::Passed => TestCaseStatus::success(),
                    TestDone::Failed => TestCaseStatus::non_success(NonSuccessKind::Failure),
                },

                _ => TestCaseStatus::non_success(NonSuccessKind::Error),
            };
            let mut test_case = TestCase::new(test.name, test_case_result);
            test_case.set_time(test.end_time - test.start_time);
            test_suite.add_test_case(test_case);
        }

        report.add_test_suite(test_suite);

        let junit_file = std::fs::File::create("junit-report.xml")?;

        report.serialize(junit_file)?;

        Ok(())
    }
}

impl Deref for TestData {
    type Target = Vec<TestMetadata>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for TestData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl std::fmt::Display for TestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InQueue => write!(f, "In Queue"),
            Self::Running(r) => write!(f, "{}", r),
            Self::Done(d) => write!(f, "{}", d),
        }
    }
}

impl std::fmt::Display for TestDone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passed => write!(f, "Passed"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

impl std::fmt::Display for TestRunning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::WaitingForInput => write!(f, "Waiting for Input"),
        }
    }
}
