use std::time::Duration;

use cli_log::*;
use color_eyre::eyre::Result;
use htf2::{Measurements, Plug, PlugEventSender, TextInput, Unit};

#[derive(Default)]
pub struct TestContext {
    pub text_input: TextInput,
    pub measurements: Measurements,
}

impl Plug for TestContext {
    fn request_sender(&mut self, sender: PlugEventSender) {
        self.text_input.request_sender(sender)
    }
}

fn test1(context: &mut TestContext) -> Result<(), htf2::TestFailure> {
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

fn test2_with_longer_name(context: &mut TestContext) -> Result<(), htf2::TestFailure> {
    info!("Running Test12");

    let input = context.text_input.request("The answer is 'Hello'");

    info!("{}", input);

    htf2::assert_eq!(input, "Hello");

    Ok(())
}

fn main() -> Result<()> {
    let (funcs, data) = htf2::register_tests!(test1, test2_with_longer_name);
    let context = TestContext::default();
    htf2::run_tests(funcs, data, context)
}
