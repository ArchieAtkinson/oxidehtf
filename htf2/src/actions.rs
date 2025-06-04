#[derive(Debug, Clone)]
pub enum Action {
    TerminalInput(crossterm::event::Event),
    SendInput,
    ExitApp,
    UserInputPrompt(String),
    UserInputValue(String),
}
