use cli_log::*;
// use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use htf2::TextInput;
// use htf2::TestContext;

pub struct TestContext {
    pub text_input: i32,
}

impl TestContext {
    fn new() -> Self {
        Self {
            text_input: Default::default(),
        }
    }
}

fn test1(context: &mut TestContext) -> Result<()> {
    info!("Running Test1");

    // let input = context.text_input.request("First Prompt");

    // let input = context.text_input.request("Second Prompt");

    info!("{}", context.text_input);

    Ok(())
}

fn main() -> Result<()> {
    let (funcs, data) = htf2::register_tests!(test1);
    let context = TestContext::new();

    htf2::run_tests(funcs, data, context)
}
