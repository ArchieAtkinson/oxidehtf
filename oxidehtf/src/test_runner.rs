use std::time::Instant;

use crate::common::*;

pub mod context;
pub mod data;
pub mod errors;
pub mod lifecycle;

pub use context::measurement::MeasurementDefinition;
pub use context::SysContext;
pub use data::current_test::CurrentTestData;
pub use data::suite::{SuiteData, SuiteDataRaw};
pub use data::{TestDone, TestRunning, TestState};
pub use errors::TestFailure;
pub use lifecycle::TestLifecycle;

pub type FuncType<T> = fn(&mut SysContext, &mut T) -> Result<(), TestFailure>;

trait SuiteTestExecuter: Send {
    fn run_test(&mut self, index: usize, context: &mut SysContext) -> Result<(), TestFailure>;

    fn fixture(&mut self) -> &mut dyn TestLifecycle;
}

#[derive(Debug, Clone)]
pub struct FunctionsAndFixture<T: TestLifecycle + Send> {
    pub functions: Vec<FuncType<T>>,
    pub fixture: T,
}

impl<T: TestLifecycle + Send> SuiteTestExecuter for FunctionsAndFixture<T> {
    fn run_test(&mut self, index: usize, context: &mut SysContext) -> Result<(), TestFailure> {
        self.functions[index](context, &mut self.fixture)
    }

    fn fixture(&mut self) -> &mut dyn TestLifecycle {
        &mut self.fixture
    }
}

pub struct TestSuite {
    executer: Box<dyn SuiteTestExecuter>,
    data: SuiteData,
}

pub struct TestRunner {
    suite: TestSuite,
    event_tx: UnboundedSender<Event>,
    context: SysContext,
}

impl TestRunner {
    pub fn new<T: TestLifecycle + 'static + Send>(
        funcs: Vec<FuncType<T>>,
        data: SuiteData,
        event_tx: UnboundedSender<Event>,
        context: SysContext,
        fixture: T,
    ) -> Self {
        let test_funcs = FunctionsAndFixture {
            functions: funcs,
            fixture,
        };
        Self {
            suite: TestSuite {
                executer: Box::new(test_funcs),
                data,
            },
            event_tx,
            context,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting Test Runner");
        self.suite.data.set_suite_start_time()?;

        self.suite.executer.fixture().setup()?;

        for (index, data) in self.suite.data.current_testdata_iter().enumerate() {
            data.set_state(TestState::Running(TestRunning::Running))?;

            self.suite.executer.fixture().before_test()?;

            let start_time = Instant::now();
            let result = self.suite.executer.run_test(index, &mut self.context);
            let test_duration = Instant::now() - start_time;

            self.suite.executer.fixture().after_test()?;

            let final_state = match result {
                Ok(_) => TestState::Done(TestDone::Passed),
                Err(_) => TestState::Done(TestDone::Failed),
            };

            data.set_state(final_state)?;
            data.set_test_duration(test_duration)?;
        }

        self.event_tx.send(Event::TestsCompleted)?;

        self.suite.executer.fixture().teardown()?;

        info!("Done");

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
