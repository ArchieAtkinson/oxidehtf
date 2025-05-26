use cli_log::*;
use color_eyre::eyre::Result;
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{
    crossterm::event::{Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, LineGauge, List, Paragraph},
    Frame,
};
use tokio::sync::mpsc;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    operator::{OperatorInput, UIOperatorComms},
    test_runner::{TestMetadata, TestState},
};

pub struct Model {
    input: Input,
    state: AppState,
    tests: Vec<TestMetadata>,
    prompt: String,
    test_recivier: mpsc::UnboundedReceiver<TestMetadata>,
    op_comms: UIOperatorComms,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Running,
    #[default]
    WaitingForInput,
    Done,
}

pub enum Message {
    ExitApp,
    TestUpdate(TestMetadata),
    OperatorInput(Event),
    SendInput,
    OperatorPrompt(String),
}

impl Model {
    pub fn new(
        test_recivier: mpsc::UnboundedReceiver<TestMetadata>,
        op_comms: UIOperatorComms,
    ) -> Self {
        Self {
            input: Default::default(),
            state: Default::default(),
            tests: Default::default(),
            prompt: String::from("No Input Required"),
            test_recivier,
            op_comms,
        }
    }

    pub async fn handle_event(&mut self) -> Result<Option<Message>> {
        let mut events = EventStream::new();
        tokio::select! {
            event = events.next() => {
                if let Some(event) = event.transpose()? {
                    if let Event::Key(key) = event {
                        return match self.state {
                            AppState::Running => match key.code {
                                KeyCode::Esc => Ok(Some(Message::ExitApp)),
                                _ => Ok(None),
                            },
                            AppState::WaitingForInput => match key.code {
                                KeyCode::Esc => Ok(Some(Message::ExitApp)),
                                KeyCode::Enter => Ok(Some(Message::SendInput)),
                                _ => Ok(Some(Message::OperatorInput(event))),
                            }

                            _ => Ok(None),
                        };
                    }
                }
            }

            test_data = self.test_recivier.recv() => {
                if let Some(data) = test_data {
                   return Ok(Some(Message::TestUpdate(data)));
                }
            }

            prompt = self.op_comms.prompt_receiver.recv() => {
                if let Some(data) = prompt {
                    return Ok(Some(Message::OperatorPrompt(data.0)));
                }

            }
        }
        Ok(None)
    }

    pub async fn update(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::ExitApp => self.state = AppState::Done,
            Message::TestUpdate(d) => self.update_tests(d).await?,
            Message::OperatorInput(e) => {
                let _ = self.input.handle_event(&e);
            }
            Message::SendInput => self.send_input()?,
            Message::OperatorPrompt(s) => self.prompt = s,
        }
        Ok(())
    }

    pub fn view(&self, frame: &mut Frame) {
        let [progress_area, header_area, input_area, messages_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .areas(frame.area());

        self.render_progress(frame, progress_area);
        self.render_help_message(frame, header_area);
        self.render_input(frame, input_area);
        self.render_messages(frame, messages_area);
    }

    fn render_progress(&self, frame: &mut Frame, area: Rect) {
        let tests_finished = self
            .tests
            .iter()
            .filter(|d| d.state == TestState::Passed || d.state == TestState::Failed)
            .count() as f64;

        let total_tests = self.tests.len() as f64;

        let mut progress: f64 = tests_finished / total_tests;
        if total_tests == 0.0 {
            progress = 0.0;
        }

        let bar = LineGauge::default()
            .filled_style(Style::new().white().on_black().bold())
            .block(Block::bordered().title("Progress"))
            .ratio(progress);
        frame.render_widget(bar, area);
    }

    fn render_help_message(&self, frame: &mut Frame, area: Rect) {
        let help_message = Line::from(match self.state {
            AppState::Running => vec![
                "Press ".to_span(),
                "Esc".bold(),
                " to exit, ".to_span(),
                "Enter".bold(),
                " to record message.".to_span(),
            ],
            AppState::WaitingForInput => vec![self.prompt.to_span()],
            AppState::Done => vec![Span::default()],
        });
        frame.render_widget(help_message, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        // keep 2 for borders and 1 for cursor
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let style = match self.state {
            AppState::WaitingForInput => Color::Yellow.into(),
            _ => Style::default(),
        };
        let input = Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, area);

        if self.state == AppState::Running {
            // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
            // end of the input text and one line down from the border to the input line
            let x = self.input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1))
        }
    }

    pub fn render_messages(&self, frame: &mut Frame, area: Rect) {
        let messages = self
            .tests
            .iter()
            .enumerate()
            .map(|(i, data)| format!("{}: {} {:?}", i, data.name, data.state));
        let messages = List::new(messages).block(Block::bordered().title("Messages"));
        frame.render_widget(messages, area);
    }

    pub async fn update_tests(&mut self, data: TestMetadata) -> Result<()> {
        let mut found = false;
        for item in self.tests.iter_mut() {
            if item.name == data.name {
                item.state = data.state.clone();
                found = true;
                break;
            }
        }
        if !found {
            self.tests.push(data);
        }
        Ok(())
    }

    pub fn mode(&self) -> AppState {
        self.state
    }

    fn send_input(&mut self) -> Result<()> {
        let input = OperatorInput(self.input.value_and_reset());
        self.op_comms.operator_sender.send(input)?;
        Ok(())
    }
}
