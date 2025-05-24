use tokio::time::Duration;

use color_eyre::Result;
use tokio::sync::mpsc;

pub struct TestRunner {
    tests: Vec<Test>,
    sender: mpsc::UnboundedSender<TestMetadata>,
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
                crate::test_runner::Test {
                    func: $func_name,
                    data: TestMetadata {
                        name: stringify!($func_name),
                        state: crate::test_runner::TestState::Waiting,
                    },
                }
            ),*
        ]
    };
}
impl TestRunner {
    pub fn new(sender: mpsc::UnboundedSender<TestMetadata>, tests: Vec<Test>) -> Result<Self> {
        for test in &tests {
            sender.send(test.data.clone())?
        }
        Ok(Self { tests, sender })
    }

    pub async fn run(&mut self) -> Result<()> {
        for test in &mut self.tests {
            test.data.state = TestState::Running;
            self.sender.send(test.data.clone())?;
            tokio::time::sleep(Duration::from_secs(2)).await;
            let result = (test.func)();
            test.data.state = match result {
                Ok(_) => TestState::Passed,
                Err(_) => TestState::Failed,
            };
            self.sender.send(test.data.clone())?
        }
        Ok(())
    }
}
