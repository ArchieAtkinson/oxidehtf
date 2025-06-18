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

pub trait SuiteTestExecuter: 'static + Send + Sync {
    fn run_test(&mut self, index: usize, context: &mut SysContext) -> Result<(), TestFailure>;

    fn fixture(&mut self) -> &mut dyn TestLifecycle;

    fn fixture_init(&mut self);
}

#[derive(Debug, Clone)]
pub struct FunctionsAndFixture<T: TestLifecycle + Send> {
    pub functions: Vec<FuncType<T>>,
    pub fixture: Option<T>,
    pub fixture_init: fn() -> T,
}

impl<T: TestLifecycle + Send> SuiteTestExecuter for FunctionsAndFixture<T> {
    fn run_test(&mut self, index: usize, context: &mut SysContext) -> Result<(), TestFailure> {
        self.functions[index](context, &mut self.fixture.as_mut().unwrap())
    }

    fn fixture(&mut self) -> &mut dyn TestLifecycle {
        self.fixture.as_mut().unwrap()
    }

    fn fixture_init(&mut self) {
        self.fixture = Some((self.fixture_init)())
    }
}

pub struct TestSuite {
    executer: Box<dyn SuiteTestExecuter>,
    data: SuiteData,
}

impl TestSuite {
    pub fn new(executer: Box<dyn SuiteTestExecuter>, data: SuiteData) -> Self {
        Self { executer, data }
    }
}

pub struct TestSuiteInventoryFactory {
    pub func: fn() -> TestSuiteInventory,
}
pub struct TestSuiteInventory {
    pub executer: Box<dyn SuiteTestExecuter>,
    pub names: Vec<&'static str>,
}

impl TestSuiteInventory {
    pub fn new<T: TestLifecycle>(
        funcs: Vec<FuncType<T>>,
        fixture_init: fn() -> T,
        names: Vec<&'static str>,
    ) -> Self {
        Self {
            executer: Box::new(FunctionsAndFixture {
                functions: funcs,
                fixture: None,
                fixture_init,
            }),
            names,
        }
    }
}

inventory::collect!(TestSuiteInventoryFactory);

pub struct TestRunner {
    suite: TestSuite,
    event_tx: UnboundedSender<Event>,
    action_rx: broadcast::Receiver<Action>,
    context: SysContext,
}

impl TestRunner {
    pub fn new(
        executer: Box<dyn SuiteTestExecuter>,
        data: SuiteData,
        event_tx: UnboundedSender<Event>,
        action_tx: broadcast::Sender<Action>,
    ) -> Self {
        Self {
            suite: TestSuite::new(executer, data.clone()),
            event_tx: event_tx.clone(),
            action_rx: action_tx.subscribe(),
            context: SysContext::new(data.clone(), event_tx, action_tx.subscribe()),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting Test Runner");

        loop {
            if let Ok(action) = self.action_rx.blocking_recv() {
                match action {
                    Action::StartTests => break,
                    _ => (),
                }
            }
        }

        info!("Starting Tests");

        self.suite.executer.fixture_init();

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
