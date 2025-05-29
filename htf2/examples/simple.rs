use cli_log::*;
// use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use htf2::TestContext;

fn test1(context: &mut TestContext) -> Result<()> {
    info!("Running Test1");

    let input = context.text_input.request("Hello");

    info!("{}", input);

    Ok(())
}

fn main() -> Result<()> {
    let tests = htf2::register_tests!(test1);
    htf2::run_tests(tests)
}
