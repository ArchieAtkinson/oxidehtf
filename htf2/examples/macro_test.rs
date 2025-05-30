#[macros::tests]
mod tests {
    use cli_log::*;
    use color_eyre::eyre::Result;
    use htf2::TestContext;

    #[test]
    fn test1(context: &mut TestContext) -> Result<()> {
        let value = context.text_input.request("Test 1 Input:");
        info!("{:?}", value);
        Ok(())
    }
}
