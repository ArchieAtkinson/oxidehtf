use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use cli_log::*;
use color_eyre::Result;
use tokio::sync::{mpsc, RwLock};

use crate::{events::Event, plugs::user_text_input::UserInput};

pub type FuncType<T> = fn(&mut T) -> Result<()>;

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
pub struct TestFunctions<T> {
    pub funcs: Vec<FuncType<T>>,
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

pub struct TestRunner<T> {
    data: Arc<RwLock<TestData>>,
    funcs: TestFunctions<T>,
    event_tx: mpsc::UnboundedSender<Event>,
    context: T,
}

impl<T> TestRunner<T> {
    pub fn new(
        funcs: TestFunctions<T>,
        data: Arc<RwLock<TestData>>,
        event_tx: mpsc::UnboundedSender<Event>,
        context: T,
    ) -> Self {
        Self {
            data,
            funcs,
            event_tx,
            context,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting Test Runner");
        let num_tests = self.data.blocking_read().data.len();

        info!("Loop");

        for index in 0..num_tests {
            self.data.blocking_write().data[index].state = TestState::Running(TestRunning::Running);

            self.event_tx.send(Event::UpdatedTestData)?;

            let result = (self.funcs.funcs[index])(&mut self.context);

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
