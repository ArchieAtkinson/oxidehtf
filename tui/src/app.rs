use color_eyre::eyre::{Context, OptionExt, Result};
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, List, Paragraph},
    Frame,
};
use tokio::sync::mpsc;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::events::{IncomingEvents, OutgoingEvents};

#[derive(Debug)]
pub struct Model {
    /// Current value of the input box
    input: Input,
    /// Current input mode
    state: AppState,
    /// History of recorded messages
    messages: Vec<String>,

    recv: mpsc::UnboundedReceiver<OutgoingEvents>,
    send: mpsc::UnboundedSender<IncomingEvents>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    #[default]
    Running,
    Done,
}

pub enum Message {
    UserInput(Event),
    ExitApp,
    SubmitInput,
}

pub async fn update(model: &mut Model, msg: Message) -> Result<()> {
    match msg {
        Message::ExitApp => model.state = AppState::Done,
        Message::SubmitInput => model.push_message().await?,
        Message::UserInput(e) => {
            let _ = model.input.handle_event(&e);
        }
    }
    Ok(())
}

pub async fn handle_event(model: &Model) -> Result<Option<Message>> {
    let mut events = EventStream::new();
    let event = events.next().await.ok_or_eyre("End of Stream")??;
    if let Event::Key(key) = event {
        return match model.state {
            AppState::Running => match key.code {
                KeyCode::Enter => Ok(Some(Message::SubmitInput)),
                KeyCode::Esc => Ok(Some(Message::ExitApp)),
                _ => Ok(Some(Message::UserInput(event))),
            },
            _ => Ok(None),
        };
    }
    Ok(None)
}

pub fn view(model: &Model, frame: &mut Frame) {
    let [header_area, input_area, messages_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(1),
    ])
    .areas(frame.area());

    render_help_message(model, frame, header_area);
    render_input(model, frame, input_area);
    render_messages(model, frame, messages_area);
}

pub fn render_help_message(model: &Model, frame: &mut Frame, area: Rect) {
    let help_message = Line::from(match model.state {
        AppState::Running => {
            vec![
                "Press ".to_span(),
                "Esc".bold(),
                " to exit, ".to_span(),
                "Enter".bold(),
                " to record message.".to_span(),
            ]
        }
        AppState::Done => vec![Span::default()],
    });
    frame.render_widget(help_message, area);
}

pub fn render_input(model: &Model, frame: &mut Frame, area: Rect) {
    // keep 2 for borders and 1 for cursor
    let width = area.width.max(3) - 3;
    let scroll = model.input.visual_scroll(width as usize);
    let style = match model.state {
        AppState::Running => Color::Yellow.into(),
        _ => Style::default(),
    };
    let input = Paragraph::new(model.input.value())
        .style(style)
        .scroll((0, scroll as u16))
        .block(Block::bordered().title("Input"));
    frame.render_widget(input, area);

    if model.state == AppState::Running {
        // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
        // end of the input text and one line down from the border to the input line
        let x = model.input.visual_cursor().max(scroll) - scroll + 1;
        frame.set_cursor_position((area.x + x as u16, area.y + 1))
    }
}

pub fn render_messages(model: &Model, frame: &mut Frame, area: Rect) {
    let messages = model
        .messages
        .iter()
        .enumerate()
        .map(|(i, message)| format!("{}: {}", i, message));
    let messages = List::new(messages).block(Block::bordered().title("Messages"));
    frame.render_widget(messages, area);
}

impl Model {
    pub fn new(
        recv: mpsc::UnboundedReceiver<OutgoingEvents>,
        send: mpsc::UnboundedSender<IncomingEvents>,
    ) -> Self {
        Self {
            input: Default::default(),
            state: Default::default(),
            recv,
            send,
            messages: Default::default(),
        }
    }
    pub async fn push_message(&mut self) -> Result<()> {
        self.send
            .send(IncomingEvents::InputRequest(self.input.value_and_reset()))
            .wrap_err("Issue")?;
        let msg = self.recv.recv().await.expect("Failed");
        match msg {
            OutgoingEvents::OperatorInput(s) => self.messages.push(s),
        }
        Ok(())
    }

    pub fn mode(&self) -> AppState {
        self.state
    }
}
