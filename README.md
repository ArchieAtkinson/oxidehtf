# OxideHTF

**NOTE: This project is very alpha and could change at any point**

OxideHTF is a Hardware Test Framework written in Rust that provides a TUI to allow operators to view and interact with tests. This project is inspired by the OpenHTF project which has the same use case.

OxideHTF provides (or will...):
- A Test Runner
- A TUI for monitoring and controlling tests
- Ability to take input from an operator
- Surfaces user-defined dependencies to each test 
- Log measurements taken and check their correctness


**Feature List:**
- General
    - Crate docs
    - User docs
    - Better examples
    - Tests
- Test Runner
    - Runs Tests ✅
    - Async Tests
    - Suite Naming
    - Multiple Suits
    - Multiple DUTs
    - Test Failures
        - Create more specific errors
        - Capture more information
    - Tests that require some or no dependencies
    - Macros
        - Gather Tests ✅
        - `#[init]` function for user fixture creation
        - Improved error messages
- Tools
    - Input over TUI
        - Text Input ✅
        - Validate Input Before Submission
            - Y/N
            - Numbers
            - Enum
    - Measurement System
        - MVP ✅
        - Display on TUI ✅
        - Units??
        - More validators
        - Transformation
    - Native Flashing Support
        - Probe-RS?
    - Native Serial Support
- TUI
    - MVP ✅
    - Control
        - Pausing
        - Restarting
            - Tests
            - Suites
        - Run
            - Selected Tests
            - Selected Suites
    - Scrollable Sections
    - Redesign?
        - More info on highlight?
        - More colours
    - Integrate WASM crate for web control
    - Welcome Screen
    - Test Data Viewer 
- Reporting
    - Junit ✅
    - Custom JSON Report with all info


```
                                  Test Suite Progress: 0% (0/2)
┌Second Prompt───────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ DUT: MyDUT - Current Test: test1 ──────────────────────────────────────────────────────────────┐
│Measurement Name                Value                            Units                          │
│First Input Value               Test                             None                           │
│A Voltage Measurement           1.5                              Volts                          │
│String Measurement              Test Value                       None                           │
│                                                                                                │
│                                                                                                │
│                                                                                                │
│                                                                                                │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
┌Completed Tests────────────────────────────────┐┌Upcoming Tests─────────────────────────────────┐
│                                               ││test1 - Waiting for Input                      │
│                                               ││test2_with_longer_name - In Queue              │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
│                                               ││                                               │
└───────────────────────────────────────────────┘└───────────────────────────────────────────────┘
```


## Example Tests

### Without Macro
```rust
#[derive(Default)]
pub struct Fixture {}

impl TestLifecycle for Fixture {}

fn test(context: &mut SysContext, _fixture: &mut Fixture) -> Result<(), htf2::TestFailure> {
    context.dut.set_via_operator(&mut context.text_input);

    let input = context.text_input.request("A example prompt to operator");
    htf2::assert_eq!(input, "Test");

    context
        .measurements
        .measure("First Input Value")
        .set_str(&input)?;

    context
        .measurements
        .measure("A Voltage Measurement")
        .with_unit("Volts")
        .in_range(0.0, 10.0)
        .set(1.5)?;

    Ok(())
}


fn main() -> Result<()> {
    let (funcs, names) = htf2::register_tests!(test);
    let context = Fixture::default();
    htf2::run_tests(funcs, names, context)
}

```

### With Macro
```rust
#[macros::tests]
mod tests {
    use htf2::{SysContext, TestLifecycle};

    #[derive(Default)]
    pub struct Fixture {}

    impl TestLifecycle for Fixture {}

    #[test]
    fn test(context: &mut SysContext, _fixture: &mut Fixture) -> Result<(), htf2::TestFailure> {
        context.dut.set_via_operator(&mut context.text_input);

        let input = context.text_input.request("A example prompt to operator");
        htf2::assert_eq!(input, "Test");

        context
            .measurements
            .measure("First Input Value")
            .set_str(&input)?;

        context
            .measurements
            .measure("A Voltage Measurement")
            .with_unit("Volts")
            .in_range(0.0, 10.0)
            .set(1.5)?;

        Ok(())
    }
}
```

## OxideHTF vs OpenHTF

The key differences between OxideHTF and OpenHTF are as follows:
- OxideHTF is written in Rust, while OpenHTF is in Python
- OxideHTF provides a TUI for interaction, while OpenHTF (primarily) provides a web interface.
- Hopefully more to come...  

One large benefit of these differences is portability. Having a Rust-based test runner allows for a test binary to be produced and distributed. This, combined with a TUI interface, provides a feature rich UI (one day ..) that can be used on almost any system.   


