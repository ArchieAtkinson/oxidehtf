// use std::time::Duration;

use std::any::Any;

use cli_log::*;
use color_eyre::eyre::Result;
use oxidehtf::{
    DynTestFn, SuiteProducer, SuiteProducerGenerator, SysContext, TestFailure, TestLifecycle,
};

struct Suite {}

impl Suite {
    fn new() -> Self {
        Self {}
    }

    fn test1(&mut self, context: &mut SysContext) -> Result<(), TestFailure> {
        context.dut.set_via_operator(&mut context.text_input)?;

        for i in 0..20 {
            context
                .measurements
                .measure(format!("Measurement {i}"))
                .set_str(format!("Value {i}"))?;
        }

        let input = context.text_input.request("The answer is 'Test'")?;

        info!("{}", input);

        oxidehtf::assert_eq!(input, "Test");

        Ok(())
    }

    fn test2(&mut self, context: &mut SysContext) -> Result<(), TestFailure> {
        let input = context.text_input.request("The answer is 'Hello'")?;

        info!("{}", input);

        oxidehtf::assert_eq!(input, "Hello");

        Ok(())
    }
}

impl TestLifecycle for Suite {}

impl SuiteProducer for Suite {
    fn get_suite_name(&self) -> &'static str {
        "suite1"
    }

    fn get_tests(&self) -> Vec<(&'static str, DynTestFn)> {
        let mut tests: Vec<(&'static str, DynTestFn)> = Vec::new();

        tests.push((
            "test1",
            Box::new(|suite_dyn, context| {
                let any_suite_dyn: &mut dyn Any = suite_dyn;
                let suite = any_suite_dyn
                    .downcast_mut::<Suite>()
                    .expect("Failed to downcast suite to Suite");
                suite.test1(context)
            }),
        ));

        tests.push((
            "test2",
            Box::new(|suite_dyn, context| {
                let any_suite_dyn: &mut dyn Any = suite_dyn;
                let suite = any_suite_dyn
                    .downcast_mut::<Suite>()
                    .expect("Failed to downcast suite to Suite");
                suite.test2(context)
            }),
        ));

        tests
    }
}

fn make_executor() -> Box<dyn SuiteProducer> {
    Box::new(Suite::new())
}

inventory::submit!(SuiteProducerGenerator {
    func: make_executor,
    prio: 0
});

fn main() -> Result<()> {
    oxidehtf::run_tests()
}
