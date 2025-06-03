#[macros::tests]
mod tests {
    use std::time::Duration;

    use cli_log::*;
    use color_eyre::eyre::Result;
    use htf2::{SysContext, TestLifecycle, Unit};

    #[derive(Default)]
    pub struct Fixture {}

    impl TestLifecycle for Fixture {}

    #[test]
    fn test1(context: &mut SysContext, _fixture: &mut Fixture) -> Result<(), htf2::TestFailure> {
        info!("Running Test1");

        let input = context.text_input.request("The answer is 'Test'");

        info!("{}", input);

        htf2::assert_eq!(input, "Test");

        std::thread::sleep(Duration::from_secs(1));

        context
            .measurements
            .measure("Test")
            .with_unit(Unit::Volts)
            .in_range(0.0, 10.0)
            .set(1.0)?;

        let input = context.text_input.request("Second Prompt");

        std::thread::sleep(Duration::from_secs(1));

        info!("{}", input);

        Ok(())
    }
}
