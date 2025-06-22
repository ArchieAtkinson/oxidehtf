use tokio::sync::oneshot;
use tui_input::InputRequest;

use crate::app::Screen;

#[derive(Debug)]
pub enum Action {
    SendInput,
    ExitApp,
    UserInputPrompt(String, Option<oneshot::Sender<String>>),
    FocusNextPane,
    FocusPreviousPane,
    MoveUp,
    MoveDown,
    UserKeyInputRequest(InputRequest),
    ChangeScreen(Screen),
    StartTests,
    SetCurrentSuiteDut(String),
}
