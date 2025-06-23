#[oxidehtf_macros::tests]
mod suite {
    use oxidehtf::SysContext;
    use oxidehtf::TestLifecycle;

    pub struct Fixture {}
    impl TestLifecycle for Fixture {}

    #[fixture]
    fn fixture() -> Fixture {
        Fixture {}
    }

    #[test]
    fn test1(_context: &mut SysContext, _fixture: &mut Fixture) -> Result<(), u32> {
        Ok(())
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    oxidehtf::run_tests()
}
