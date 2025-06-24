use cli_log::*;
use oxidehtf::SysContext;
use oxidehtf::TestFailure;
use oxidehtf::TestLifecycle;

struct Suite {}

#[oxidehtf_macros::tests(1)]
impl Suite {
    fn new() -> Self {
        Self {}
    }

    #[test]
    fn test1(&mut self, context: &mut SysContext) -> Result<(), TestFailure> {
        context.dut.set_via_operator(&mut context.text_input)?;

        for i in 0..20 {
            context
                .measurements
                .measure(format!("Measurement {i}"))
                .set_str(format!("Value {i}"))?;
        }

        let input = context.text_input.request("The answer is 'Test'")?;

        info!("{}", input);

        oxidehtf::assert_eq!(input, "Test");

        Ok(())
    }

    #[test]
    fn test2(&mut self, context: &mut SysContext) -> Result<(), TestFailure> {
        let input = context.text_input.request("The answer is 'Hello'")?;

        info!("{}", input);

        oxidehtf::assert_eq!(input, "Hello");

        Ok(())
    }
}

impl TestLifecycle for Suite {}

struct Suite2 {}

#[oxidehtf_macros::tests(1)]
impl Suite2 {
    fn new() -> Self {
        Self {}
    }

    #[test]
    fn test1(&mut self, _context: &mut SysContext) -> Result<(), TestFailure> {
        Ok(())
    }
}

impl TestLifecycle for Suite2 {}

fn main() -> color_eyre::eyre::Result<()> {
    oxidehtf::run_tests()
}
