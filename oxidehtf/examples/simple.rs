// use std::time::Duration;

use cli_log::*;
use color_eyre::eyre::Result;
use oxidehtf::{SysContext, TestLifecycle};

#[derive(Default)]
pub struct Fixture {}

impl TestLifecycle for Fixture {}

fn fixture() -> Fixture {
    Fixture {}
}

fn test1(context: &mut SysContext, _fixture: &mut Fixture) -> Result<(), oxidehtf::TestFailure> {
    context.dut.set_via_operator(&mut context.text_input);

    let input = context.text_input.request("The answer is 'Test'");

    info!("{}", input);

    oxidehtf::assert_eq!(input, "Test");

    context
        .measurements
        .measure("A Voltage Measurement")
        .with_unit("Volts")
        .in_range(0.0, 10.0)
        .set(1.5)?;

    for i in 0..20 {
        context
            .measurements
            .measure(format!("Measurement {i}"))
            .set_str(format!("Value {i}"))?;
    }

    let input = context.text_input.request("Second Prompt");

    info!("{}", input);

    Ok(())
}

fn test2_with_longer_name(
    context: &mut SysContext,
    _fixture: &mut Fixture,
) -> Result<(), oxidehtf::TestFailure> {
    let input = context.text_input.request("The answer is 'Hello'");

    info!("{}", input);

    oxidehtf::assert_eq!(input, "Hello");

    Ok(())
}

fn create_suite_1() -> oxidehtf::TestSuiteBuilder {
    oxidehtf::TestSuiteBuilder::new(
        vec![test1, test2_with_longer_name],
        fixture,
        vec!["test1", "test2"],
        "Suite2",
    )
}

fn create_suite_2() -> oxidehtf::TestSuiteBuilder {
    oxidehtf::TestSuiteBuilder::new(
        vec![test1, test2_with_longer_name],
        fixture,
        vec!["test1", "test2"],
        "Suite1",
    )
}

inventory::submit! {
    oxidehtf::TestSuiteBuilderProducer {func: create_suite_1}
}

inventory::submit! {
    oxidehtf::TestSuiteBuilderProducer {func: create_suite_2}
}

fn main() -> Result<()> {
    oxidehtf::run_tests()
}
