#[derive(Debug, Clone)]
pub enum Action {
    TerminalInput(crossterm::event::Event),
    SendInput,
    ExitApp,
    OperatorTextInput(String),
}
