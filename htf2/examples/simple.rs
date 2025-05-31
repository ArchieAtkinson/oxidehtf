use std::time::Duration;

use cli_log::*;
// use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use htf2::{Event, Plug, TextInput};
use tokio::sync::mpsc::UnboundedSender;

pub struct TestContext {
    pub text_input: TextInput,
}

impl Plug for TestContext {
    fn register_event_handler(&mut self, tx: UnboundedSender<Event>) -> Result<()> {
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

fn test1(context: &mut TestContext) -> Result<()> {
    info!("Running Test1");

    let input = context.text_input.request("First Prompt");

    info!("{}", input);

    std::thread::sleep(Duration::from_secs(1));

    let input = context.text_input.request("Second Prompt");

    info!("{}", input);

    Ok(())
}

fn main() -> Result<()> {
    let (funcs, data) = htf2::register_tests!(test1);
    let context = TestContext::new();
    htf2::run_tests(funcs, data, context)
}
