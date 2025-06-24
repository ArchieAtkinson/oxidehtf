use std::any::Any;

use crate::common::*;

use super::{SysContext, TestFailure, TestLifecycle};

pub type DynTestFn =
    Box<dyn Fn(&mut dyn SuiteProducer, &mut SysContext) -> Result<(), TestFailure> + Send + Sync>;

pub trait SuiteProducer: TestLifecycle + Send + Sync + Any {
    fn get_tests(&self) -> Vec<(&'static str, DynTestFn)>;
    fn get_suite_name(&self) -> &'static str;
}

pub struct SuiteProducerGenerator {
    pub func: fn() -> Box<dyn SuiteProducer>,
    pub prio: usize,
}
