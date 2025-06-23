#[oxidehtf_macros::tests]
mod suite {
    use oxidehtf::SysContext;
    use oxidehtf::TestFailure;
    use oxidehtf::TestLifecycle;

    pub struct Fixture {}
    impl TestLifecycle for Fixture {}

    #[fixture]
    fn fixture() -> Fixture {
        Fixture {}
    }

    #[test]
    fn test1(_context: SysContext, _fixture: &mut Fixture) -> Result<(), TestFailure> {
        Ok(())
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    oxidehtf::run_tests()
}
