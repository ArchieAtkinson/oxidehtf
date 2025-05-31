#[derive(Debug, Clone)]
pub enum Action {
    TerminalInput(crossterm::event::Event),
    SendInput,
    ExitApp,
    UserInputValue(String),
    UserInputPrompt(String),
}
