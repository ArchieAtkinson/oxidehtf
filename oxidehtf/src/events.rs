use crossterm::event::{KeyEvent, MouseEvent};
use tokio::sync::oneshot;

pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    UpdatedTestData,
    TestsCompleted,
    UserInputPrompt(String, Option<oneshot::Sender<String>>),
    CurrentSuiteDut(String),
}
