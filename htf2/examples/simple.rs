use cli_log::*;
// use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use htf2::TestContext;

fn test1(context: &mut TestContext) -> Result<()> {
    info!("Running Test1");

    let input = context.text_input.request("First Prompt");

    let input = context.text_input.request("Second Prompt");

    info!("{}", input);

    Ok(())
}

fn main() -> Result<()> {
    let (funcs, data) = htf2::register_tests!(test1);

    htf2::run_tests(funcs, data)
}
