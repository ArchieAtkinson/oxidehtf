#[derive(Clone, Debug, PartialEq)]
pub enum TestFailure {
    AssertionFailed {
        expected: String,
        found: String,
        file: &'static str,
        line: u32,
    },
    MeasurementNotInRange {
        name: String,
        expected: (f64, f64),
        found: f64,
        file: &'static str,
        line: u32,
    },
    MeasurementDoesntExist(String),
    SystemExited,
    SystemError,
}

impl std::fmt::Display for TestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssertionFailed {
                expected,
                found,
                file,
                line,
            } => write!(
                f,
                "Assertion Failed at {}:{} - Expected: {} - Found: {}",
                file, line, expected, found
            ),
            Self::MeasurementNotInRange {
                name,
                expected,
                found,
                file: _,
                line: _,
            } => write!(
                f,
                "Measurement '{}' - Expected: {}-{} - Found: {}",
                name, expected.0, expected.1, found,
            ),
            Self::MeasurementDoesntExist(name) => write!(f, "Measurement '{}' doesn't exist", name),
            Self::SystemExited => write!(f, "System Exited"),
            Self::SystemError => write!(f, "System Failed and Exited"),
        }
    }
}
