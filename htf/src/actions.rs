#[derive(Debug, Clone)]
pub enum Action {
    OperatorInput(crossterm::event::Event),
    SendInput,
    OperatorPrompt(String),
    ExitApp,
}
