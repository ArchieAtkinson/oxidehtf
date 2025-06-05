use std::time::Instant;

use cli_log::*;
use color_eyre::Result;
use context::SysContext;
use errors::TestFailure;
use lifecycle::TestLifecycle;
use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite};
use test_data::{TestDataManager, TestDone, TestRunning, TestState};
use tokio::sync::mpsc;

use crate::events::Event;

pub mod context;
pub mod errors;
pub mod lifecycle;
pub mod test_data;

pub type FuncType<T> = fn(&mut SysContext, &mut T) -> Result<(), TestFailure>;

#[derive(Debug, Clone)]
pub struct TestFunctions<T> {
    pub funcs: Vec<FuncType<T>>,
}

pub struct TestRunner<T: TestLifecycle> {
    data_manager: TestDataManager,
    funcs: TestFunctions<T>,
    event_tx: mpsc::UnboundedSender<Event>,
    context: SysContext,
    fixture: T,
}

impl<T: TestLifecycle> TestRunner<T> {
    pub fn new(
        funcs: TestFunctions<T>,
        data: TestDataManager,
        event_tx: mpsc::UnboundedSender<Event>,
        context: SysContext,
        fixture: T,
    ) -> Self {
        Self {
            data_manager: data,
            funcs,
            event_tx,
            context,
            fixture,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting Test Runner");
        let num_tests = self.data_manager.blocking_read(|d| Ok(d.len()))?;

        info!("Loop");

        self.fixture.setup()?;

        for index in 0..num_tests {
            self.data_manager.blocking_write(|d| {
                d.current_index = index;
                d.current_test_mut().state = TestState::Running(TestRunning::Running);
                d.current_test_mut().start_time = Instant::now();
                Ok(())
            })?;

            self.fixture.before_test()?;
            let result = (self.funcs.funcs[index])(&mut self.context, &mut self.fixture);
            self.fixture.after_test()?;

            self.data_manager.blocking_write(|d| {
                d.test_metadata[index].state = match result {
                    Ok(_) => TestState::Done(TestDone::Passed),
                    Err(e) => {
                        error!("{:#?}", e);
                        TestState::Done(TestDone::Failed)
                    }
                };
                d.current_test_mut().end_time = Instant::now();
                Ok(())
            })?;
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

        let data = self.data_manager.blocking_get_copy();

        for test in data.test_metadata {
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
