use crate::components::test_runner::TestMetadata;

#[derive(Debug, Clone)]
pub enum Action {
    TerminalInput(crossterm::event::Event),
    SendInput,
    OperatorPrompt(String),
    ExitApp,
    TestUpdate(TestMetadata),
}
