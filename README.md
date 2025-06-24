# OxideHTF

**NOTE: This project is very alpha and could change at any point**

OxideHTF is a Hardware Test Framework written in Rust that provides a TUI to allow operators to view and interact with tests. This project is inspired by the OpenHTF project which has the same use case.

OxideHTF provides (or will...):
- A Test Runner
- A TUI for monitoring and controlling tests
- Ability to take input from an operator
- Surfaces user-defined dependencies to each test 
- Log measurements taken and check their correctness

**Know Bugs**


**Feature List:**
- General
    - Crate docs
    - User docs
    - Better examples
    - Tests
- Test Runner
    - Runs Tests ✅
    - Async Tests
    - Suite Naming ✅
    - Multiple Suits ✅
        - Set Suite Order ✅
    - Multiple DUTs
    - Test Failures
        - Create more specific errors
        - Capture more information
    - Tests that require some or no dependencies
    - Macros
        - Gather Tests ✅
        - Improved error messages
        - Move to Object system instead of Modules ✅
    - Test Rules
        - End suite on failure
        - End Test Run on failure
        - Asserts that can fail
        - Measurements that can fail
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
    - Component Focusing
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
        - MVP ✅
    - Redesign?
        - More info on highlight?
        - More colours
    - Integrate WASM crate for web control
    - Welcome Screen ✅
    - Summary Screen
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
└────────────────────────────────────────────────────────────────────────────────────────────────┘
┌Completed Tests────────────────────────────────┐┌Upcoming Tests─────────────────────────────────┐
│                                               ││test1 - Waiting for Input                      │
│                                               ││test2_with_longer_name - In Queue              │
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

fn test(context: &mut SysContext, _fixture: &mut Fixture) -> Result<(), oxidehtf::TestFailure> {
    context.dut.set_via_operator(&mut context.text_input);

    let input = context.text_input.request("A example prompt to operator");
    oxidehtf::assert_eq!(input, "Test");

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
    let (funcs, names) = oxidehtf::register_tests!(test);
    let context = Fixture::default();
    oxidehtf::run_tests(funcs, names, context)
}

```

### With Macro
```rust
#[macros::tests]
mod tests {
    use oxidehtf::{SysContext, TestLifecycle};

    #[derive(Default)]
    pub struct Fixture {}

    impl TestLifecycle for Fixture {}

    #[test]
    fn test(context: &mut SysContext, _fixture: &mut Fixture) -> Result<(), oxidehtf::TestFailure> {
        context.dut.set_via_operator(&mut context.text_input);

        let input = context.text_input.request("A example prompt to operator");
        oxidehtf::assert_eq!(input, "Test");

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

## Architecture Musings

### Events and Actions/Commands

#### Events
Events are raw inputs
Events should be produced by components (and others?) and produce actions
Events shouldn't change state or take actions

#### Actions
Actions should be used to change state or do work

