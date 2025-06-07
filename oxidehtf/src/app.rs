use cli_log::*;
use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use tokio::sync::mpsc;

use crate::{
    actions::Action,
    components::{test_status::TestStatusDisplay, user_text_input::UserTextInput, Component},
    events::Event,
    test_runner::test_data::TestDataManager,
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
    test_data: TestDataManager,
    components: Vec<Box<dyn Component>>,
    current_focus: usize,
    action_rx: mpsc::UnboundedReceiver<Action>,
    action_tx: mpsc::UnboundedSender<Action>,
    event_rx: mpsc::UnboundedReceiver<Event>,
    event_tx: mpsc::UnboundedSender<Event>,
    input_tx: mpsc::UnboundedSender<String>,
}

impl App {
    pub fn new(
        test_data: TestDataManager,
        event_rx: mpsc::UnboundedReceiver<Event>,
        event_tx: mpsc::UnboundedSender<Event>,
        input_tx: mpsc::UnboundedSender<String>,
    ) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        Ok(Self {
            ui: Ui::new(event_tx.clone()),
            test_data,
            components: vec![
                Box::new(UserTextInput::new()),
                Box::new(TestStatusDisplay::new()),
            ],
            current_focus: 0,
            state: Default::default(),
            action_rx,
            action_tx,
            event_rx,
            event_tx,
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

        self.components[self.current_focus].focus();

        info!("Spawning Test Runner");

        while self.state() != AppState::Done {
            self.handle_event().await?;
            self.handle_actions().await?;
            let state = self.test_data.get_copy().await;
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
                        KeyCode::Tab => {
                            self.focus_next();
                        }
                        _ => {}
                    }
                }
                self.action_tx
                    .send(Action::TerminalInput(crossterm_event))?;
            }
            Event::UserInputPrompt(s) => {
                self.action_tx.send(Action::UserInputPrompt(s))?;
            }
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
                Action::UserInputValue(s) => {
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

    fn focus_next(&mut self) {
        self.components[self.current_focus].blur();

        let len = self.components.len();

        let start_search_index = self.current_focus + 1;
        let mut next_focus_index = 0;

        let mut found_next_focusable = false;

        for i in 0..len {
            let index = (start_search_index + i) % len;
            if self.components[index].can_focus() {
                next_focus_index = index;
                found_next_focusable = true;
                break;
            }
        }

        if found_next_focusable {
            self.components[next_focus_index].focus();
            self.current_focus = next_focus_index;
        } else {
            panic!(
                "No other focusable components found in the sequence. Current focus remains {}.",
                self.current_focus
            );
        }
    }
}
