use crate::common::*;

use super::{FuncType, SysContext, TestFailure, TestLifecycle};

pub trait SuiteExecuter: 'static + Send + Sync {
    fn run_test(&mut self, index: usize, context: &mut SysContext) -> Result<(), TestFailure>;

    fn fixture(&mut self) -> &mut dyn TestLifecycle;

    fn fixture_init(&mut self);
}

#[derive(Debug, Clone)]
pub struct SuiteExecuterHolder<T: TestLifecycle + Send> {
    pub functions: Vec<FuncType<T>>,
    pub fixture: Option<T>,
    pub fixture_init: fn() -> T,
}

impl<T: TestLifecycle> SuiteExecuterHolder<T> {
    pub fn new(functions: Vec<FuncType<T>>, fixture_init: fn() -> T) -> Self {
        Self {
            functions,
            fixture: None,
            fixture_init,
        }
    }
}

impl<T: TestLifecycle + Send> SuiteExecuter for SuiteExecuterHolder<T> {
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
