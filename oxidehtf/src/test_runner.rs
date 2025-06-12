use std::time::Instant;

use cli_log::*;
use color_eyre::Result;
use context::SysContext;
use errors::TestFailure;
use lifecycle::TestLifecycle;
use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite};
use tokio::sync::mpsc;

use crate::events::Event;
use crate::test_data::{SuiteData, TestDone, TestRunning, TestState};

pub mod context;
pub mod errors;
pub mod lifecycle;

pub type FuncType<T> = fn(&mut SysContext, &mut T) -> Result<(), TestFailure>;

trait SuiteTestRunner: Send {
    fn run_test(&mut self, index: usize, context: &mut SysContext) -> Result<(), TestFailure>;

    fn fixture(&mut self) -> &mut dyn TestLifecycle;
}

#[derive(Debug, Clone)]
pub struct ProvidedData<T: TestLifecycle + Send> {
    pub functions: Vec<FuncType<T>>,
    pub fixture: T,
}

impl<T: TestLifecycle + Send> SuiteTestRunner for ProvidedData<T> {
    fn run_test(&mut self, index: usize, context: &mut SysContext) -> Result<(), TestFailure> {
        self.functions[index](context, &mut self.fixture)
    }

    fn fixture(&mut self) -> &mut dyn TestLifecycle {
        &mut self.fixture
    }
}

pub struct TestRunner {
    data: SuiteData,
    suite_funcs: Box<dyn SuiteTestRunner>,
    event_tx: mpsc::UnboundedSender<Event>,
    context: SysContext,
}

impl TestRunner {
    pub fn new<T: TestLifecycle + 'static + Send>(
        funcs: Vec<FuncType<T>>,
        data: SuiteData,
        event_tx: mpsc::UnboundedSender<Event>,
        context: SysContext,
        fixture: T,
    ) -> Self {
        let test_funcs = ProvidedData {
            functions: funcs,
            fixture,
        };
        Self {
            data,
            suite_funcs: Box::new(test_funcs),
            event_tx,
            context,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting Test Runner");
        self.data.set_suite_start_time()?;

        self.suite_funcs.fixture().setup()?;

        for (index, data) in self.data.current_testdata_iter().enumerate() {
            data.set_state(TestState::Running(TestRunning::Running))?;

            self.suite_funcs.fixture().before_test()?;

            let start_time = Instant::now();
            let result = self.suite_funcs.run_test(index, &mut self.context);
            let test_duration = Instant::now() - start_time;

            self.suite_funcs.fixture().after_test()?;

            let final_state = match result {
                Ok(_) => TestState::Done(TestDone::Passed),
                Err(_) => TestState::Done(TestDone::Failed),
            };

            data.set_state(final_state)?;
            data.set_test_duration(test_duration)?;
        }

        self.event_tx.send(Event::TestsCompleted)?;

        self.suite_funcs.fixture().teardown()?;

        info!("Done");

        self.produce_junit_report()?;

        Ok(())
    }

    fn produce_junit_report(&self) -> Result<()> {
        let mut report = Report::new("htf2-run");
        let mut test_suite = TestSuite::new("htf2-suite");

        let data = self.data.blocking_get_raw_copy();

        for test in data.test_metadata {
            let test_case_result = match test.state {
                TestState::Done(r) => match r {
                    TestDone::Passed => TestCaseStatus::success(),
                    TestDone::Failed => TestCaseStatus::non_success(NonSuccessKind::Failure),
                },

                _ => TestCaseStatus::non_success(NonSuccessKind::Error),
            };
            let mut test_case = TestCase::new(test.name, test_case_result);
            test_case.set_time(test.duration);
            test_suite.add_test_case(test_case);
        }

        report.add_test_suite(test_suite);
        report.timestamp = Some(data.start_time);

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
