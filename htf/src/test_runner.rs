use color_eyre::{eyre::eyre, Result};
use tokio::sync::mpsc;

pub struct TestRunner {
    tests: Vec<Test>,
    test_sender: mpsc::UnboundedSender<TestMetadata>,
    operator_comms: OperatorComms,
}

pub struct Test {
    pub func: fn(&mut OperatorComms) -> Result<()>,
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

#[derive(Debug)]
pub struct OperatorPrompt(pub String);

#[derive(Debug)]
pub struct OperatorInput(pub String);

#[macro_export]
macro_rules! register_test {
    ($($func_name:ident),*) => {
        vec![
            $(
                htf::test_runner::Test {
                    func: $func_name,
                    data: TestMetadata {
                        name: stringify!($func_name),
                        state: htf::test_runner::TestState::Waiting,
                    },
                }
            ),*
        ]
    };
}

impl TestRunner {
    pub fn new(
        test_sender: mpsc::UnboundedSender<TestMetadata>,
        prompt_sender: mpsc::UnboundedSender<OperatorPrompt>,
        operator_recivier: mpsc::UnboundedReceiver<OperatorInput>,
        tests: Vec<Test>,
    ) -> Result<Self> {
        for test in &tests {
            test_sender.send(test.data.clone())?
        }
        Ok(Self {
            tests,
            test_sender,
            operator_comms: OperatorComms {
                prompt_sender,
                operator_recivier,
            },
        })
    }

    pub fn run(&mut self) -> Result<()> {
        for test in &mut self.tests {
            test.data.state = TestState::Running;
            self.test_sender.send(test.data.clone())?;
            let result = (test.func)(&mut self.operator_comms);
            test.data.state = match result {
                Ok(_) => TestState::Passed,
                Err(_) => TestState::Failed,
            };
            self.test_sender.send(test.data.clone())?
        }
        Ok(())
    }
}

pub struct OperatorComms {
    prompt_sender: mpsc::UnboundedSender<OperatorPrompt>,
    operator_recivier: mpsc::UnboundedReceiver<OperatorInput>,
}

impl OperatorComms {
    pub fn request_input(&mut self, prompt: String) -> Result<String> {
        self.prompt_sender.send(OperatorPrompt(prompt));
        let OperatorInput(input) = self
            .operator_recivier
            .blocking_recv()
            .ok_or(eyre!("Failed to get input"))?;
        Ok(input)
    }
}
