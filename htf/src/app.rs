// use cli_log::*;
use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use tokio::sync::mpsc;

use crate::{
    actions::Action,
    components::{test_runner::TestRunner, Component},
    events::Event,
    ui::Ui,
    Input, Test,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    #[default]
    WaitingForInput,
    Done,
}

pub struct App {
    ui: Ui,
    state: AppState,
    components: Vec<Box<dyn Component>>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    action_tx: mpsc::UnboundedSender<Action>,
    event_rx: mpsc::UnboundedReceiver<Event>,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl App {
    pub fn new(tests: Vec<Test>) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Ok(Self {
            ui: Ui::new(event_tx.clone()),
            components: vec![Box::new(Input::new()), Box::new(TestRunner::new(tests))],
            state: Default::default(),
            action_rx,
            action_tx,
            event_rx,
            event_tx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        for component in self.components.iter_mut() {
            component.register_event_handler(self.event_tx.clone())?;
            component.register_action_handler(self.action_tx.clone())?;
            component.init()?;
        }

        self.ui.start();

        while self.state() != AppState::Done {
            self.handle_event().await?;
            self.handle_actions()?;
            self.ui.render(|f, a| {
                for component in self.components.iter_mut() {
                    component.draw(f, &a)?;
                }
                Ok(())
            })?;
        }

        Ok(())
    }

    async fn handle_event(&mut self) -> Result<()> {
        let Some(event) = self.event_rx.recv().await else {
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
                        .send(Action::TerminalInput(crossterm_event))?;
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

    fn state(&self) -> AppState {
        self.state
    }
}
