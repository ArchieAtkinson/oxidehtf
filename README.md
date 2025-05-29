# HTuiF


TODO:

Split out the test runner from the UI components
Create a test context system to allow "plugs" to be passed into tests
Change input into a plug.

example
```rust
// Current

#[test]
fn test1() -> Result<()> {
    let value = Input::request("Test 1 Input:");
    info!("{:?}", value);
    Ok(())
}

// to

struct TestContext {
  user_input: UserInput;
}

#[test]
fn test1(context: &mut TestContext) -> Result<()> {
    let value = context.input.request("Test 1 Input:");
    info!("{:?}", value);
    Ok(())
}
```

Things to work out.
- How is TextContext constructed?
- Implement so kind of TestLifecycle trait that provides setup, teardown etc that can be implemented for "plugs"
- Comms between the "plugs" and the UI
