use tui_input::InputRequest;

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
}
