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
pub use executer::SuiteProducer;
pub use executer::SuiteProducerGenerator;
pub use lifecycle::TestLifecycle;

inventory::collect!(SuiteProducerGenerator);

pub struct TestRunner {
    executor: Vec<Box<dyn SuiteProducer>>,
    data: SuiteDataCollection,
    event_tx: UnboundedSender<Event>,
    context: SysContext,
    from_app_rx: UnboundedReceiver<Action>,
}

impl TestRunner {
    pub fn new(
        executor: Vec<Box<dyn SuiteProducer>>,
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

        for (suite_index, executor) in self.executor.iter_mut().enumerate() {
            info!(
                "Starting Suite: {}",
                self.data.data.blocking_read().inner[suite_index].name
            );

            self.data.data.blocking_write().current = suite_index;

            self.data.blocking_write(|f| f.set_suite_start_time())?;

            executor.setup()?;

            for (test_index, (name, test)) in executor.get_tests().iter_mut().enumerate() {
                self.data.blocking_write(|f| {
                    f.current_suite_mut().update_test_index(test_index);
                    f.current_suite_mut().current_test_mut().state =
                        TestState::Running(TestRunning::Running);
                    Ok(())
                })?;

                info!("Starting Test: {}", name);

                executor.before_test()?;
                let start_time = Instant::now();
                let result = test(executor.as_mut(), &mut self.context);
                let test_duration = Instant::now() - start_time;
                executor.after_test()?;

                let final_state = match result {
                    Ok(_) => TestState::Done(TestDone::Passed),
                    Err(e) => TestState::Done(TestDone::Failed(e)),
                };

                self.data.blocking_write(|f| {
                    f.current_suite_mut().current_test_mut().state = final_state;
                    f.current_suite_mut().current_test_mut().duration = test_duration;
                    Ok(())
                })?;
            }

            self.event_tx.send(Event::TestsCompleted)?;

            executor.teardown()?;

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
            Self::Failed(_) => write!(f, "Failed"),
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
