use std::sync::OnceLock;

use color_eyre::eyre::{eyre, OptionExt, Result};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::ToLine,
    widgets::{Block, Paragraph},
    Frame,
};
use tokio::sync::{mpsc, Mutex};
use tui_input::backend::crossterm::EventHandler;

use crate::{actions::Action, app::UiArea, events::Event};

use super::{test_runner::TestState, Component};

static OPERATOR_COMMS: OnceLock<Mutex<TestOperatorComms>> = OnceLock::new();

#[derive(Debug)]
pub struct OperatorInput(pub String);

struct TestOperatorComms {
    prompt_tx: mpsc::UnboundedSender<Event>,
    operator_rx: mpsc::UnboundedReceiver<OperatorInput>,
}

const DEFAULT_PROMPT_TEXT: &'static str = "No Input Currently Required";

pub struct Input {
    action_tx: Option<mpsc::UnboundedSender<Action>>,
    event_tx: Option<mpsc::UnboundedSender<Event>>,
    input_tx: Option<mpsc::UnboundedSender<OperatorInput>>,
    txt_input: tui_input::Input,
    prompt_text: String,
}

impl Input {
    pub fn new() -> Self {
        Self {
            action_tx: Default::default(),
            event_tx: Default::default(),
            input_tx: Default::default(),
            txt_input: Default::default(),
            prompt_text: DEFAULT_PROMPT_TEXT.to_string(),
        }
    }

    pub fn request(prompt: impl Into<String>) -> Result<String> {
        let mut comms = OPERATOR_COMMS
            .get()
            .expect("Failed to get oncelock")
            .blocking_lock();

        comms.prompt_tx.send(Event::OperatorPrompt(prompt.into()))?;

        let OperatorInput(input) = comms
            .operator_rx
            .blocking_recv()
            .ok_or_eyre("Failed to get input")?;

        Ok(input)
    }

    fn draw_prompt(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let help_message = self.prompt_text.to_line();
        frame.render_widget(help_message, area);
        Ok(())
    }

    fn draw_input(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // keep 2 for borders and 1 for cursor
        let width = area.width.max(3) - 3;
        let scroll = self.txt_input.visual_scroll(width as usize);
        let style = Style::default();
        let input = Paragraph::new(self.txt_input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, area);

        // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past
        // the end of the input text and one line down from the border to the input line
        let x = self.txt_input.visual_cursor().max(scroll) - scroll + 1;
        frame.set_cursor_position((area.x + x as u16, area.y + 1));
        Ok(())
    }
}

impl Component for Input {
    fn init(&mut self) -> Result<()> {
        let (input_tx, input_rx) = mpsc::unbounded_channel::<OperatorInput>();

        self.input_tx = Some(input_tx);

        let Some(prompt_tx) = self.event_tx.take() else {
            return Err(eyre!("No Prompt Tx"));
        };

        OPERATOR_COMMS
            .set(Mutex::new(TestOperatorComms {
                prompt_tx,
                operator_rx: input_rx,
            }))
            .map_err(|_| eyre!("Failed to init Operator Comms"))?;

        Ok(())
    }

    fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx.clone());
        Ok(())
    }

    fn register_event_handler(&mut self, tx: mpsc::UnboundedSender<Event>) -> Result<()> {
        self.event_tx = Some(tx.clone());
        Ok(())
    }

    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        match event {
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::OperatorInput(e) => {
                let _ = self.txt_input.handle_event(&e);
                Ok(None)
            }
            Action::SendInput => {
                let input = OperatorInput(self.txt_input.value_and_reset());
                self.input_tx
                    .as_ref()
                    .ok_or_eyre("Failed to get input tx")?
                    .send(input)?;
                Ok(None)
            }
            Action::OperatorPrompt(p) => {
                self.prompt_text = p;
                Ok(None)
            }
            Action::TestUpdate(d) => {
                if d.state == TestState::Passed || d.state == TestState::Failed {
                    self.prompt_text = DEFAULT_PROMPT_TEXT.into();
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: &UiArea) -> Result<()> {
        assert_eq!(area.operator.height, 4);

        let [prompt_area, input_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(3)]).areas(area.operator);

        self.draw_prompt(frame, prompt_area)?;
        self.draw_input(frame, input_area)?;
        Ok(())
    }
}
