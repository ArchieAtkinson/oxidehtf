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
    info!("Running Test1");

    // context.dut.set_via_operator(&mut context.text_input);

    // let input = context.text_input.request("The answer is 'Test'");

    // info!("{}", input);

    // oxidehtf::assert_eq!(input, "Test");

    // context
    //      .measurements
    //      .measure("First Input Value")
    //      .set_str(&input)?;

    // std::thread::sleep(Duration::from_secs(1));

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

    // std::thread::sleep(Duration::from_secs(2));

    info!("{}", input);

    Ok(())
}

fn test2_with_longer_name(
    context: &mut SysContext,
    _fixture: &mut Fixture,
) -> Result<(), oxidehtf::TestFailure> {
    info!("Running Test12");

    let input = context.text_input.request("The answer is 'Hello'");

    info!("{}", input);

    oxidehtf::assert_eq!(input, "Hello");

    Ok(())
}

fn create_suite_inventory() -> oxidehtf::TestSuiteInventory {
    oxidehtf::TestSuiteInventory::new(
        vec![test1, test2_with_longer_name],
        fixture,
        vec!["test1", "test2"],
    )
}

inventory::submit! {
    oxidehtf::TestSuiteInventoryFactory {func: create_suite_inventory}
}

fn main() -> Result<()> {
    oxidehtf::run_tests()
}
