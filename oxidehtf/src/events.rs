use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    UpdatedTestData,
    TestsCompleted,
    UserInputPrompt(String),
}
