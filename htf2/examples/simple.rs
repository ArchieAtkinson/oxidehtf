use std::time::Duration;

use cli_log::*;
use color_eyre::eyre::Result;
use htf2::{SysContext, TestLifecycle};

#[derive(Default)]
pub struct Fixture {}

impl TestLifecycle for Fixture {}

fn test1(context: &mut SysContext, _fixture: &mut Fixture) -> Result<(), htf2::TestFailure> {
    info!("Running Test1");

    context.dut.set_via_operator(&mut context.text_input);

    let input = context.text_input.request("The answer is 'Test'");

    info!("{}", input);

    htf2::assert_eq!(input, "Test");

    context
        .measurements
        .measure("First Input Value")
        .set_str(&input)?;

    std::thread::sleep(Duration::from_secs(1));

    context
        .measurements
        .measure("A Voltage Measurement")
        .with_unit("Volts")
        .in_range(0.0, 10.0)
        .set(1.5)?;

    context
        .measurements
        .measure("String Measurement")
        .set_str("Test Value")?;

    let input = context.text_input.request("Second Prompt");

    std::thread::sleep(Duration::from_secs(2));

    info!("{}", input);

    Ok(())
}

fn test2_with_longer_name(
    context: &mut SysContext,
    _fixture: &mut Fixture,
) -> Result<(), htf2::TestFailure> {
    info!("Running Test12");

    let input = context.text_input.request("The answer is 'Hello'");

    info!("{}", input);

    htf2::assert_eq!(input, "Hello");

    Ok(())
}

fn main() -> Result<()> {
    let (funcs, names) = htf2::register_tests!(test1, test2_with_longer_name);
    let context = Fixture::default();
    htf2::run_tests(funcs, names, context)
}
