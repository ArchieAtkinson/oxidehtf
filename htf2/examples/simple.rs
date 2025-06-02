use std::time::Duration;

use cli_log::*;
use color_eyre::eyre::Result;
use htf2::{SysContext, TestLifecycle, Unit};

#[derive(Default)]
pub struct Fixture {}

impl TestLifecycle for Fixture {}

fn test1(context: &mut SysContext, _fixture: &mut Fixture) -> Result<(), htf2::TestFailure> {
    info!("Running Test1");

    let input = context.text_input.request("The answer is 'Test'");

    info!("{}", input);

    htf2::assert_eq!(input, "Test");

    std::thread::sleep(Duration::from_secs(1));

    let input = context.text_input.request("Second Prompt");

    info!("{}", input);

    context
        .measurements
        .measure("Test")
        .with_unit(Unit::Volts)
        .in_range(0.0, 10.0)
        .set(11.0)?;

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
    let (funcs, data) = htf2::register_tests!(test1, test2_with_longer_name);
    let context = Fixture::default();
    htf2::run_tests(funcs, data, context)
}
