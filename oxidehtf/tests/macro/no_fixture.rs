#[oxidehtf_macros::tests]
mod suite {
    use oxidehtf::SysContext;

    #[test]
    fn test1(_context: &mut SysContext) -> Result<(), oxidehtf::TestFailure> {
        Ok(())
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    oxidehtf::run_tests()
}
