use std::time::Instant;

use crate::common::*;

pub mod context;
pub mod data;
pub mod errors;
pub mod executer;
pub mod lifecycle;

pub use context::measurement::MeasurementDefinition;
pub use context::SysContext;
pub use data::current_test::CurrentTestData;
use data::suite::SuiteDataCollection;
pub use data::suite::SuiteDataRaw;
pub use data::{TestDone, TestRunning, TestState};
pub use errors::TestFailure;
pub use executer::SuiteExecuter;
pub use executer::SuiteExecuterHolder;
pub use lifecycle::TestLifecycle;

pub type FuncType<T> = fn(&mut SysContext, &mut T) -> Result<(), TestFailure>;

pub struct TestSuiteBuilderProducer {
    pub func: fn() -> TestSuiteBuilder,
}
pub struct TestSuiteBuilder {
    pub executer: Box<dyn SuiteExecuter>,
    pub data: SuiteDataRaw,
}

impl TestSuiteBuilder {
    pub fn new<T: TestLifecycle>(
        funcs: Vec<FuncType<T>>,
        fixture_init: fn() -> T,
        names: Vec<&'static str>,
    ) -> Self {
        Self {
            executer: Box::new(SuiteExecuterHolder::new(funcs, fixture_init)),
            data: SuiteDataRaw::new(names),
        }
    }
}

inventory::collect!(TestSuiteBuilderProducer);

pub struct TestRunner {
    executor: Vec<Box<dyn SuiteExecuter>>,
    data: SuiteDataCollection,
    event_tx: UnboundedSender<Event>,
    action_rx: broadcast::Receiver<Action>,
    context: SysContext,
}

impl TestRunner {
    pub fn new(
        executor: Vec<Box<dyn SuiteExecuter>>,
        data: SuiteDataCollection,
        event_tx: UnboundedSender<Event>,
        action_tx: broadcast::Sender<Action>,
    ) -> Self {
        Self {
            executor,
            data: data.clone(),
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
            } else {
                return Err(eyre!("Failed to start tests"));
            }
        }

        let suite_num = self.data.data.blocking_read().inner.len();
        for suite_index in 0..suite_num {
            info!("Starting Suite: {}", suite_index);

            self.data.data.blocking_write().current = suite_index;

            self.executor[suite_index].fixture_init();

            self.data.set_suite_start_time()?;

            self.executor[suite_index].fixture().setup()?;

            for (index, data) in self.data.current_testdata_iter().enumerate() {
                info!("Starting Test: {}", index);

                data.set_state(TestState::Running(TestRunning::Running))?;

                self.executor[suite_index].fixture().before_test()?;

                let start_time = Instant::now();
                let result = self.executor[suite_index].run_test(index, &mut self.context);
                let test_duration = Instant::now() - start_time;

                self.executor[suite_index].fixture().after_test()?;

                let final_state = match result {
                    Ok(_) => TestState::Done(TestDone::Passed),
                    Err(_) => TestState::Done(TestDone::Failed),
                };

                data.set_state(final_state)?;
                data.set_test_duration(test_duration)?;
            }

            self.event_tx.send(Event::TestsCompleted)?;

            self.executor[suite_index].fixture().teardown()?;

            info!("Done");
        }
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
