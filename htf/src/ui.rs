use cli_log::*;
use color_eyre::eyre::Result;
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, LineGauge, List},
    Frame,
};
use tokio::sync::mpsc;

use crate::{
    actions::Action,
    component::Component,
    events::Event,
    operator::Input,
    test_runner::{TestMetadata, TestState},
};

pub struct Ui {
    state: AppState,
    tests: Vec<TestMetadata>,
    test_recivier: mpsc::UnboundedReceiver<TestMetadata>,
    op_input: Input,
    action_rx: mpsc::UnboundedReceiver<Action>,
    action_tx: mpsc::UnboundedSender<Action>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Running,
    #[default]
    WaitingForInput,
    Done,
}

impl Ui {
    pub fn new(test_recivier: mpsc::UnboundedReceiver<TestMetadata>) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let mut op_input = Input::new()?;
        op_input.register_action_handler(action_tx.clone())?;
        Ok(Self {
            op_input,
            state: Default::default(),
            tests: Default::default(),
            test_recivier,
            action_rx,
            action_tx,
        })
    }

    pub async fn handle_event(&mut self) -> Result<Option<Event>> {
        let mut events = EventStream::new();
        tokio::select! {
            event = events.next() => {
                if let Some(event) = event.transpose()? {
                    if let crossterm::event::Event::Key(key) = event {
                        return match self.state {
                            AppState::Running => match key.code {
                                KeyCode::Esc => Ok(Some(Event::ExitApp)),
                                _ => Ok(None),
                            },
                            AppState::WaitingForInput => match key.code {
                                KeyCode::Esc => Ok(Some(Event::ExitApp)),
                                KeyCode::Enter => Ok(Some(Event::SendInput)),
                                _ => Ok(Some(Event::OperatorInput(event))),
                            }

                            _ => Ok(None),
                        };
                    }
                }
            }

            test_data = self.test_recivier.recv() => {
                if let Some(data) = test_data {
                   return Ok(Some(Event::TestUpdate(data)));
                }
            }
        }
        Ok(None)
    }

    pub async fn update(&mut self, event: Event) -> Result<()> {
        match event.clone() {
            Event::ExitApp => self.state = AppState::Done,
            Event::TestUpdate(d) => self.update_tests(d).await?,
            _ => (),
        }
        if let Some(action) = self.op_input.handle_events(Some(event))? {
            self.action_tx.send(action)?;
        };

        if let Ok(action) = self.action_rx.try_recv() {
            if let Some(new_action) = self.op_input.update(action)? {
                self.action_tx.send(new_action)?;
            }
        }
        Ok(())
    }

    pub fn view(&mut self, frame: &mut Frame) -> Result<()> {
        let [progress_area, op_area, messages_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(1),
        ])
        .areas(frame.area());

        self.render_progress(frame, progress_area);
        self.op_input.draw(frame, op_area)?;
        self.render_messages(frame, messages_area);
        Ok(())
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
}
