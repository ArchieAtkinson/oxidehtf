// use cli_log::*;
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
    event_rx: mpsc::UnboundedReceiver<Event>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    #[default]
    WaitingForInput,
    Done,
}

impl Ui {
    pub fn new(test_recivier: mpsc::UnboundedReceiver<TestMetadata>) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let mut op_input = Input::new(event_tx.clone())?;
        op_input.register_action_handler(action_tx.clone())?;
        Ok(Self {
            op_input,
            state: Default::default(),
            tests: Default::default(),
            test_recivier,
            action_rx,
            action_tx,
            event_rx,
        })
    }

    pub async fn handle_event(&mut self) -> Result<()> {
        let mut events = EventStream::new();

        let event = tokio::select! {
            crossterm = events.next() => {
                crossterm.transpose()?.map(|e| Event::CrosstermEvent(e))

            }
            test_data = self.test_recivier.recv() => {
                test_data.map(|d| Event::TestData(d))
            }
            external = self.event_rx.recv() => {
                external
            }

        };

        let Some(event) = event else {
            return Ok(());
        };

        match event {
            Event::CrosstermEvent(event) => {
                if let crossterm::event::Event::Key(key) = event {
                    if key.code == KeyCode::Esc {
                        self.action_tx.send(Action::ExitApp)?;
                    }

                    match self.state {
                        AppState::WaitingForInput => match key.code {
                            KeyCode::Enter => self.action_tx.send(Action::SendInput)?,
                            _ => self.action_tx.send(Action::OperatorInput(event))?,
                        },
                        _ => (),
                    };
                }
            }
            Event::TestData(d) => self.action_tx.send(Action::TestUpdate(d))?,
            Event::OperatorPrompt(p) => self.action_tx.send(Action::OperatorPrompt(p))?,
        }

        Ok(())
    }

    pub fn handle_actions(&mut self) -> Result<()> {
        if let Ok(action) = self.action_rx.try_recv() {
            match action.clone() {
                Action::TestUpdate(d) => self.update_tests(d)?,
                Action::ExitApp => self.state = AppState::Done,
                Action::OperatorPrompt(_) => self.state = AppState::WaitingForInput,
                _ => (),
            }
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

    pub fn update_tests(&mut self, data: TestMetadata) -> Result<()> {
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
