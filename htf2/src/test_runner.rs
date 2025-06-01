use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use cli_log::*;
use color_eyre::Result;
use tokio::sync::{mpsc, RwLock};

use crate::{events::Event, plugs::user_text_input::UserInput, TestFailure};

pub type FuncType<T> = fn(&mut T) -> Result<(), TestFailure>;

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
            {
                let lock = &mut self.data.blocking_write();
                lock.data[index].state = TestState::Running(TestRunning::Running);
                lock.current_index = index;
            }

            self.event_tx.send(Event::UpdatedTestData)?;

            let result = (self.funcs.funcs[index])(&mut self.context);

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

impl std::fmt::Display for TestMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}\n", self.name, self.state)?;
        if !self.user_inputs.is_empty() {
            write!(f, "     Operator Input(s)")?;
            for input in self.user_inputs.clone() {
                write!(f, "\n        Prompt: {}", input.prompt)?;
                if !input.input.is_empty() {
                    write!(f, "\n        Input: {}\n", input.input)?;
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for UserInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.input.is_empty() {
            return write!(f, "Prompt: {}", self.prompt);
        } else {
            return write!(f, "Prompt: {}\nInput: {}", self.prompt, self.input);
        }
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
