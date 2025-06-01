use std::time::Duration;

use cli_log::*;
use color_eyre::eyre::Result;
use htf2::{Event, Plug, TextInput};
use tokio::sync::mpsc::UnboundedSender;

pub struct TestContext {
    pub text_input: TextInput,
}

impl Plug for TestContext {
    fn register_event_handler(&mut self, tx: UnboundedSender<Event>) {
        self.text_input.register_event_handler(tx)
    }
}

impl TestContext {
    fn new() -> Self {
        Self {
            text_input: TextInput::new(),
        }
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
    let context = TestContext::new();
    htf2::run_tests(funcs, data, context)
}
