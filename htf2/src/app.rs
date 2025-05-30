use std::sync::Arc;

use cli_log::*;
use color_eyre::eyre::{OptionExt, Result};
use crossterm::event::KeyCode;
use tokio::sync::{mpsc, RwLock};

use crate::{
    actions::Action,
    components::{test_status::TestStatusDisplay, user_text_input::UserTextInput, Component},
    events::Event,
    test_runner::{TestData, TestFunctions, TestRunner},
    ui::Ui,
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
    test_data: Arc<RwLock<TestData>>,
    test_funcs: TestFunctions,
    components: Vec<Box<dyn Component>>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    action_tx: mpsc::UnboundedSender<Action>,
    event_rx: mpsc::UnboundedReceiver<Event>,
    event_tx: mpsc::UnboundedSender<Event>,
    input_rx: Option<mpsc::UnboundedReceiver<String>>,
    input_tx: mpsc::UnboundedSender<String>,
}

impl App {
    pub fn new(funcs: TestFunctions, data: TestData) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (input_tx, input_rx) = mpsc::unbounded_channel();

        Ok(Self {
            ui: Ui::new(event_tx.clone()),
            test_data: Arc::new(RwLock::new(data)),
            test_funcs: funcs,
            components: vec![
                Box::new(TestStatusDisplay::new()),
                Box::new(UserTextInput::new()),
            ],
            state: Default::default(),
            action_rx,
            action_tx,
            event_rx,
            event_tx,
            input_rx: Some(input_rx),
            input_tx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        for component in self.components.iter_mut() {
            component.register_event_handler(self.event_tx.clone())?;
            component.register_action_handler(self.action_tx.clone())?;
            component.init()?;
        }

        self.ui.start();

        // TODO: Handle Errors
        let mut test_runner = TestRunner::new(
            self.test_funcs.clone(),
            self.test_data.clone(),
            self.event_tx.clone(),
        );

        info!("Spawning Test Runner");

        let input_rx = self.input_rx.take().ok_or_eyre("No Input RX")?;

        tokio::task::spawn_blocking(move || test_runner.run(input_rx));

        while self.state() != AppState::Done {
            self.handle_event().await?;
            self.handle_actions().await?;
            let state = self.test_data.read().await;
            self.ui.render(|f, a| {
                for component in self.components.iter_mut() {
                    component.draw(f, &a, &state)?;
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
            // Event::UpdatedTestRunnerState =
            _ => (),
        }

        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(event.clone())? {
                self.action_tx.send(action)?;
            }
        }

        Ok(())
    }

    async fn handle_actions(&mut self) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            match action.clone() {
                Action::ExitApp => self.state = AppState::Done,
                Action::OperatorTextInput(s) => {
                    self.input_tx.send(s)?;
                }
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
