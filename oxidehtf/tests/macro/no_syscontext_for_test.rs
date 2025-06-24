use oxidehtf::TestFailure;
use oxidehtf::TestLifecycle;

struct Suite {}

#[oxidehtf_macros::tests]
impl Suite {
    fn new() -> Self {
        Self {}
    }

    #[test]
    fn test1(&mut self, _foo: u32) -> Result<(), TestFailure> {
        Ok(())
    }
}

impl TestLifecycle for Suite {}

fn main() -> color_eyre::eyre::Result<()> {
    oxidehtf::run_tests()
}
