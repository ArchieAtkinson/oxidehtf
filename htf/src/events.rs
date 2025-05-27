use crate::components::test_runner::TestMetadata;

#[derive(Debug, Clone)]
pub enum Event {
    CrosstermEvent(crossterm::event::Event),
    OperatorPrompt(String),
    TestData(TestMetadata),
}
