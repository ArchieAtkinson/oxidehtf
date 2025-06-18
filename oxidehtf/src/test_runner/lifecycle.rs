use crate::common::*;

pub trait TestLifecycle: 'static + Send + Sync {
    fn setup(&mut self) -> Result<()> {
        Ok(())
    }

    fn before_test(&mut self) -> Result<()> {
        Ok(())
    }

    fn after_test(&mut self) -> Result<()> {
        Ok(())
    }

    fn teardown(&mut self) -> Result<()> {
        Ok(())
    }
}
