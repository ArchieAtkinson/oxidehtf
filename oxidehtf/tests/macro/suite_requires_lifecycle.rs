use oxidehtf::SysContext;
use oxidehtf::TestFailure;

struct Suite {}

#[oxidehtf_macros::tests]
impl Suite {
    fn new() -> Self {
        Self {}
    }

    #[test]
    fn test1(&mut self, _context: &mut SysContext) -> Result<(), TestFailure> {
        Ok(())
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    oxidehtf::run_tests()
}
