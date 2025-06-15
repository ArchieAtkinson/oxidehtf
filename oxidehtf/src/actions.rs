use tui_input::InputRequest;

use crate::app::Screen;

#[derive(Debug, Clone)]
pub enum Action {
    SendInput,
    ExitApp,
    UserInputPrompt(String),
    UserInputValue(String),
    FocusNextPane,
    FocusPreviousPane,
    MoveUp,
    MoveDown,
    InputRequest(InputRequest),
    ChangeScreen(Screen),
}
