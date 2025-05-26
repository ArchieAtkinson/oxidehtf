use crate::test_runner::TestMetadata;

#[derive(Debug, Clone)]
pub enum Event {
    ExitApp,
    TestUpdate(TestMetadata),
    OperatorInput(crossterm::event::Event),
    SendInput,
}
