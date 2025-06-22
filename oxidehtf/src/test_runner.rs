use std::time::Instant;

use crate::common::*;

pub mod context;
pub mod data;
pub mod errors;
pub mod executer;
pub mod lifecycle;

pub use context::measurement::MeasurementDefinition;
pub use context::SysContext;
pub use data::suite::SuiteData;
use data::suite::SuiteDataCollection;
pub use data::suite::SuiteDataCollectionRaw;
pub use data::{TestDone, TestRunning, TestState};
pub use errors::TestFailure;
pub use executer::SuiteExecuter;
pub use executer::SuiteExecuterHolder;
pub use lifecycle::TestLifecycle;

pub type FuncType<T> = fn(&mut SysContext, &mut T) -> Result<(), TestFailure>;

pub struct TestSuiteBuilderProducer {
    pub(crate) func: fn() -> TestSuiteBuilder,
}

impl TestSuiteBuilderProducer {
    pub const fn new(func: fn() -> TestSuiteBuilder) -> Self {
        Self { func }
    }
}

pub struct TestSuiteBuilder {
    pub(crate) executer: Box<dyn SuiteExecuter>,
    pub(crate) data: SuiteData,
}

impl TestSuiteBuilder {
    pub fn new<T: TestLifecycle>(
        funcs: Vec<FuncType<T>>,
        fixture_init: fn() -> T,
        names: Vec<&'static str>,
        suite_name: &'static str,
        priority: usize,
    ) -> Self {
        Self {
            executer: Box::new(SuiteExecuterHolder::new(funcs, fixture_init)),
            data: SuiteData::new(names, suite_name, priority),
        }
    }
}

inventory::collect!(TestSuiteBuilderProducer);

pub struct TestRunner {
    executor: Vec<Box<dyn SuiteExecuter>>,
    data: SuiteDataCollection,
    event_tx: UnboundedSender<Event>,
    context: SysContext,
    from_app_rx: UnboundedReceiver<Action>,
}

impl TestRunner {
    pub fn new(
        executor: Vec<Box<dyn SuiteExecuter>>,
        data: SuiteDataCollection,
        event_tx: UnboundedSender<Event>,
        from_app_rx: UnboundedReceiver<Action>,
    ) -> Self {
        Self {
            executor,
            data: data.clone(),
            event_tx: event_tx.clone(),
            context: SysContext::new(data.clone(), event_tx),
            from_app_rx,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting Test Runner");

        loop {
            let action = self.from_app_rx.blocking_recv().unwrap();
            match action {
                Action::StartTests => break,
                _ => (),
            }
        }

        let suite_num = self.data.data.blocking_read().inner.len();
        for suite_index in 0..suite_num {
            info!(
                "Starting Suite: {}",
                self.data.data.blocking_read().inner[suite_index].name
            );

            self.data.data.blocking_write().current = suite_index;

            self.executor[suite_index].fixture_init();

            self.data.blocking_write(|f| f.set_suite_start_time())?;

            self.executor[suite_index].fixture().setup()?;

            let test_num = self
                .data
                .blocking_read(|f| Ok(f.current_suite().get_test_amount()))?;

            for index in 0..test_num {
                let test_name = self.data.blocking_write(|f| {
                    f.current_suite_mut().update_test_index(index);
                    f.current_suite_mut().current_test_mut().state =
                        TestState::Running(TestRunning::Running);
                    Ok(f.current_suite().current_test().name)
                })?;

                info!("Starting Test: {}", test_name);
                self.executor[suite_index].fixture().before_test()?;
                let start_time = Instant::now();
                let result = self.executor[suite_index].run_test(index, &mut self.context);
                let test_duration = Instant::now() - start_time;
                self.executor[suite_index].fixture().after_test()?;

                let final_state = match result {
                    Ok(_) => TestState::Done(TestDone::Passed),
                    Err(_) => TestState::Done(TestDone::Failed),
                };

                self.data.blocking_write(|f| {
                    f.current_suite_mut().current_test_mut().state = final_state;
                    f.current_suite_mut().current_test_mut().duration = test_duration;
                    Ok(())
                })?;
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
