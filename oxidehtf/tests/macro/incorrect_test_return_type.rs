use oxidehtf::SysContext;
use oxidehtf::TestLifecycle;

struct Suite {}

#[oxidehtf_macros::tests]
impl Suite {
    fn new() -> Self {
        Self {}
    }

    #[test]
    fn test1(&mut self, _context: &mut SysContext) -> Result<(), u32> {
        Ok(())
    }
}

impl TestLifecycle for Suite {}

fn main() -> color_eyre::eyre::Result<()> {
    oxidehtf::run_tests()
}
