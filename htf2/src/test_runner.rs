use std::sync::Arc;

use cli_log::*;
use color_eyre::Result;
use tokio::sync::{mpsc, RwLock};

use crate::{events::Event, TextInput};

pub(crate) mod user_text_input;

pub struct TestContext {
    pub text_input: TextInput,
}

pub struct TestRunnerState {
    pub tests: Vec<Test>,
    pub current_index: usize,
}

impl TestRunnerState {
    pub fn new(tests: Vec<Test>) -> Self {
        Self {
            tests,
            current_index: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Test {
    pub func: fn(&mut TestContext) -> Result<()>,
    pub data: TestMetadata,
}

impl Test {
    pub fn new(func: fn(&mut TestContext) -> Result<()>, name: &'static str) -> Self {
        Self {
            func,
            data: TestMetadata {
                name,
                state: Default::default(),
                user_inputs: Vec::new(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserInput {
    pub prompt: String,
    pub input: String,
}

impl UserInput {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            input: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestMetadata {
    pub name: &'static str,
    pub state: TestState,
    pub user_inputs: Vec<UserInput>,
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

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum TestState {
    #[default]
    InQueue,
    Running(TestRunning),
    Done(TestDone),
}

pub struct TestRunner {
    state: Arc<RwLock<TestRunnerState>>,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl TestRunner {
    pub fn new(
        state: Arc<RwLock<TestRunnerState>>,
        event_tx: mpsc::UnboundedSender<Event>,
    ) -> Self {
        Self { state, event_tx }
    }

    pub fn run(&mut self, input_rx: mpsc::UnboundedReceiver<String>) -> Result<()> {
        info!("Starting Test Runner");
        let num_tests = self.state.blocking_read().tests.len();

        info!("Loop");

        let mut context = TestContext {
            text_input: TextInput::new(self.state.clone(), self.event_tx.clone(), input_rx),
        };

        for index in 0..num_tests {
            let test_func = {
                let mut state_lock = self.state.blocking_write();
                let test = &mut state_lock.tests[index];
                test.data.state = TestState::Running(TestRunning::Running);
                test.func
            };

            self.event_tx.send(Event::UpdatedTestRunnerState)?;

            let result = (test_func)(&mut context);

            self.state.blocking_write().tests[index].data.state = match result {
                Ok(_) => TestState::Done(TestDone::Passed),
                Err(_) => TestState::Done(TestDone::Failed),
            };

            self.event_tx.send(Event::UpdatedTestRunnerState)?;
        }

        info!("Done");

        Ok(())
    }
}
