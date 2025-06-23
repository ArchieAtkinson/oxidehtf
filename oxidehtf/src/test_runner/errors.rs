#[derive(Debug)]
pub enum TestFailure {
    AssertionFailed {
        expected: String,
        found: String,
        file: &'static str,
        line: u32,
    },
    TextInputError(String),
    MeasurementError,
    SystemExited,
}
