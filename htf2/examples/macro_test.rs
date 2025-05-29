#[macros::tests]
mod tests {
    use cli_log::*;
    use color_eyre::eyre::{eyre, Result};
    use htf::Input;

    #[test]
    fn test1() -> Result<()> {
        let value = Input::request("Test 1 Input:");
        info!("{:?}", value);
        Ok(())
    }

    #[test]
    fn test2() -> Result<()> {
        let value = Input::request("Test 2 Input:");
        info!("{:?}", value);
        Err(eyre!("Err"))
    }

    #[test]
    fn test3() -> Result<()> {
        let value = Input::request("Test 3 Input:");
        info!("{:?}", value);
        Ok(())
    }

    #[test]
    fn test4() -> Result<()> {
        let value = Input::request("Test 4 Input:");
        info!("{:?}", value);
        Ok(())
    }
}
