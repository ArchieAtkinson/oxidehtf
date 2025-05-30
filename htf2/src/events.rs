#[derive(Debug, Clone)]
pub enum Event {
    CrosstermEvent(crossterm::event::Event),
    UpdatedTestData,
}
