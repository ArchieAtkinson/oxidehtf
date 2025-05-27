use crate::components::test_runner::TestMetadata;

#[derive(Debug, Clone)]
pub enum Action {
    OperatorInput(crossterm::event::Event),
    SendInput,
    OperatorPrompt(String),
    ExitApp,
    TestUpdate(TestMetadata),
}
