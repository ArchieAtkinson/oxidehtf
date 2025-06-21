#[oxidehtf_macros::tests]
mod suite1 {
    use std::time::Duration;

    use cli_log::*;
    use color_eyre::eyre::Result;
    use oxidehtf::{SysContext, TestLifecycle};

    pub struct Fixture {}

    impl TestLifecycle for Fixture {}

    #[fixture]
    fn fixture() -> Fixture {
        Fixture {}
    }

    #[test]
    fn test1(
        context: &mut SysContext,
        _fixture: &mut Fixture,
    ) -> Result<(), oxidehtf::TestFailure> {
        info!("Running Test1");

        let input = context.text_input.request("The answer is 'Test'");

        info!("{}", input);

        oxidehtf::assert_eq!(input, "Test");

        std::thread::sleep(Duration::from_secs(1));

        context
            .measurements
            .measure("Test")
            .with_unit("V")
            .in_range(0.0, 10.0)
            .set(1.0)?;

        let input = context.text_input.request("Second Prompt");

        std::thread::sleep(Duration::from_secs(1));

        info!("{}", input);

        Ok(())
    }
}

#[oxidehtf_macros::tests]
mod suite2 {
    use std::time::Duration;

    use cli_log::*;
    use color_eyre::eyre::Result;
    use oxidehtf::{SysContext, TestLifecycle};

    pub struct Fixture {}

    impl TestLifecycle for Fixture {}

    #[fixture]
    fn fixture() -> Fixture {
        Fixture {}
    }

    #[test]
    fn test1(
        context: &mut SysContext,
        _fixture: &mut Fixture,
    ) -> Result<(), oxidehtf::TestFailure> {
        info!("Running Test1");

        let input = context.text_input.request("The answer is 'Test'");

        info!("{}", input);

        oxidehtf::assert_eq!(input, "Test");

        std::thread::sleep(Duration::from_secs(1));

        context
            .measurements
            .measure("Test")
            .with_unit("V")
            .in_range(0.0, 10.0)
            .set(1.0)?;

        let input = context.text_input.request("Second Prompt");

        std::thread::sleep(Duration::from_secs(1));

        info!("{}", input);

        Ok(())
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    oxidehtf::run_tests()
}
