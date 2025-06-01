#[derive(Debug, Clone)]
pub enum Event {
    CrosstermEvent(crossterm::event::Event),
    UpdatedTestData,
    TestsCompleted,
    PlugEvent(PlugEvent),
}

#[derive(Debug, Clone)]
pub enum PlugEvent {
    UserInputPrompt(String),
}
