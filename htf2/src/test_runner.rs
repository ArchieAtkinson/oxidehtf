use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use cli_log::*;
use color_eyre::Result;
use context::{measurement::MeasurementDefinition, SysContext};
use errors::TestFailure;
use indexmap::IndexMap;
use lifecycle::TestLifecycle;
use tokio::sync::{mpsc, RwLock};

use crate::events::Event;

pub mod context;
pub mod errors;
pub mod lifecycle;

pub type FuncType<T> = fn(&mut SysContext, &mut T) -> Result<(), TestFailure>;

#[derive(Debug, Clone)]
pub struct TestData {
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
            drop(data_guard);

            self.event_tx.send(Event::UpdatedTestData)?;

            self.fixture.before_test()?;
            let result = (self.funcs.funcs[index])(&mut self.context, &mut self.fixture);
            self.fixture.after_test()?;

            self.data.blocking_write()[index].state = match result {
                Ok(_) => TestState::Done(TestDone::Passed),
                Err(e) => {
                    error!("{:#?}", e);
                    TestState::Done(TestDone::Failed)
                }
            };

            self.event_tx.send(Event::UpdatedTestData)?;
        }

        self.event_tx.send(Event::TestsCompleted)?;

        self.fixture.teardown()?;

        info!("Done");

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

// impl std::fmt::Display for TestMetadata {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{} - {}", self.name, self.state)?;
//         if self.user_data.is_empty() {
//             return Ok(());
//         }
//         for (key, value) in self.user_data.clone().iter().rev() {
//             match value {
//                 UserDataType::Input(i) => {
//                     write!(f, "\n     Operator Input")?;
//                     write!(f, "\n        Prompt: {}", key)?;
//                     if !i.is_empty() {
//                         write!(f, "\n        Input: {}\n", i)?;
//                     } else {
//                         write!(f, "\n        Input: <Waiting For Input>\n")?;
//                     }
//                 }
//                 UserDataType::Measurement(m) => {
//                     write!(f, "\n     Measurement")?;
//                     write!(f, "\n        Name: {}", key)?;

//                     if let Some(value) = &m.value {
//                         write!(f, "\n        Input: {}\n", value)?;
//                     } else {
//                         write!(f, "\n        Input: <Waiting For Input>\n")?;
//                     }
//                 }
//             }
//         }
//         Ok(())
//     }
// }

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
