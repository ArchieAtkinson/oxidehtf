use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use cli_log::*;
use color_eyre::Result;
use tokio::sync::{mpsc, RwLock};

use crate::{events::Event, TextInput};

pub(crate) mod user_text_input;

pub type FuncType = fn(&mut TestContext) -> Result<()>;

pub struct TestContext {
    pub text_input: TextInput,
}

#[derive(Debug, Clone)]
pub struct TestData {
    pub data: Vec<TestMetadata>,
    pub current_index: usize,
}

#[derive(Debug, Clone)]
pub struct TestMetadata {
    pub name: &'static str,
    pub state: TestState,
    pub user_inputs: Vec<UserInput>,
}

#[derive(Debug, Clone)]
pub struct TestFunctions {
    pub funcs: Vec<FuncType>,
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
    data: Arc<RwLock<TestData>>,
    funcs: TestFunctions,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl TestRunner {
    pub fn new(
        funcs: TestFunctions,
        data: Arc<RwLock<TestData>>,
        event_tx: mpsc::UnboundedSender<Event>,
    ) -> Self {
        Self {
            data,
            funcs,
            event_tx,
        }
    }

    pub fn run(&mut self, input_rx: mpsc::UnboundedReceiver<String>) -> Result<()> {
        info!("Starting Test Runner");
        let num_tests = self.data.blocking_read().data.len();

        info!("Loop");

        let mut context = TestContext {
            text_input: TextInput::new(self.data.clone(), self.event_tx.clone(), input_rx),
        };

        for index in 0..num_tests {
            self.data.blocking_write().data[index].state = TestState::Running(TestRunning::Running);

            self.event_tx.send(Event::UpdatedTestData)?;

            let result = (self.funcs[index])(&mut context);

            self.data.blocking_write()[index].state = match result {
                Ok(_) => TestState::Done(TestDone::Passed),
                Err(_) => TestState::Done(TestDone::Failed),
            };

            self.event_tx.send(Event::UpdatedTestData)?;
        }

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

impl Deref for TestFunctions {
    type Target = Vec<FuncType>;

    fn deref(&self) -> &Self::Target {
        &self.funcs
    }
}

impl DerefMut for TestFunctions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.funcs
    }
}
