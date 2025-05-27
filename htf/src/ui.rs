use cli_log::*;
use color_eyre::eyre::Result;
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc;

use crate::{
    actions::Action,
    components::{
        operator::Input,
        test_runner::{Test, TestRunner},
        Component,
    },
    events::Event,
};

pub struct UiArea {
    pub test_progress: Rect,
    pub operator: Rect,
    pub test_list: Rect,
}

pub struct Ui {
    state: AppState,
    components: Vec<Box<dyn Component>>,
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
    pub fn new(tests: Vec<Test>) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let mut op_input = Input::new(event_tx.clone())?;
        op_input.register_action_handler(action_tx.clone())?;

        let mut test_runner = TestRunner::new(tests, event_tx)?;
        test_runner.register_action_handler(action_tx.clone())?;

        Ok(Self {
            components: vec![Box::new(op_input), Box::new(test_runner)],
            state: Default::default(),
            action_rx,
            action_tx,
            event_rx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
            component.init();
        }

        let mut terminal = ratatui::init();

        info!("Run");

        while self.mode() != AppState::Done {
            let event = self.next_event().await?;
            self.handle_event(event)?;
            self.handle_actions()?;

            let mut result = Ok(());
            terminal.draw(|f| result = self.view(f))?;
            if result.is_err() {
                return result;
            }
        }

        ratatui::restore();
        Ok(())
    }

    async fn next_event(&mut self) -> Result<Option<Event>> {
        let mut events = EventStream::new();

        Ok(tokio::select! {
            crossterm = events.next() => {
                crossterm.transpose()?.map(|e| Event::CrosstermEvent(e))
            }
            external = self.event_rx.recv() => {
                external
            }
        })
    }

    fn handle_event(&mut self, event: Option<Event>) -> Result<()> {
        let Some(event) = event else {
            return Ok(());
        };

        match event.clone() {
            Event::CrosstermEvent(crossterm_event) => {
                if let crossterm::event::Event::Key(key) = crossterm_event {
                    match key.code {
                        KeyCode::Esc => {
                            self.action_tx.send(Action::ExitApp)?;
                        }
                        KeyCode::Enter if self.state == AppState::WaitingForInput => {
                            self.action_tx.send(Action::SendInput)?;
                        }
                        _ => {}
                    }
                }

                if self.state == AppState::WaitingForInput {
                    self.action_tx
                        .send(Action::OperatorInput(crossterm_event))?;
                }
            }
            Event::OperatorPrompt(p) => self.action_tx.send(Action::OperatorPrompt(p))?,
            _ => (),
        }

        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(event.clone())? {
                self.action_tx.send(action)?;
            }
        }

        Ok(())
    }

    fn handle_actions(&mut self) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            match action.clone() {
                Action::ExitApp => self.state = AppState::Done,
                Action::OperatorPrompt(_) => self.state = AppState::WaitingForInput,
                _ => (),
            }

            for component in self.components.iter_mut() {
                if let Some(new_action) = component.update(action.clone())? {
                    self.action_tx.send(new_action)?;
                }
            }
        }

        Ok(())
    }

    fn view(&mut self, frame: &mut Frame) -> Result<()> {
        let [test_progress, operator, test_list] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(1),
        ])
        .areas(frame.area());

        let areas = UiArea {
            test_progress,
            operator,
            test_list,
        };

        for component in self.components.iter_mut() {
            component.draw(frame, &areas)?;
        }

        Ok(())
    }

    fn mode(&self) -> AppState {
        self.state
    }
}
