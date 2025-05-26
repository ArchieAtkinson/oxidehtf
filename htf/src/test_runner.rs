use color_eyre::Result;
use tokio::sync::mpsc;

pub struct TestRunner {
    tests: Vec<Test>,
    test_sender: mpsc::UnboundedSender<TestMetadata>,
}

pub struct Test {
    pub func: fn() -> Result<()>,
    pub data: TestMetadata,
}

#[derive(Clone)]
pub struct TestMetadata {
    pub name: &'static str,
    pub state: TestState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestState {
    Waiting,
    Running,
    Passed,
    Failed,
}

#[macro_export]
macro_rules! register_test {
    ($($func_name:ident),*) => {
        vec![
            $(
                htf::test_runner::Test {
                    func: $func_name,
                    data: htf::test_runner::TestMetadata {
                        name: stringify!($func_name),
                        state: htf::test_runner::TestState::Waiting,
                    },
                }
            ),*
        ]
    };
}

impl TestRunner {
    pub fn new(test_sender: mpsc::UnboundedSender<TestMetadata>, tests: Vec<Test>) -> Result<Self> {
        for test in &tests {
            test_sender.send(test.data.clone())?
        }
        Ok(Self { tests, test_sender })
    }

    pub fn run(&mut self) -> Result<()> {
        for test in &mut self.tests {
            test.data.state = TestState::Running;
            self.test_sender.send(test.data.clone())?;
            let result = (test.func)();
            test.data.state = match result {
                Ok(_) => TestState::Passed,
                Err(_) => TestState::Failed,
            };
            self.test_sender.send(test.data.clone())?
        }
        Ok(())
    }
}
