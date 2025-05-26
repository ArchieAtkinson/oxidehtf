use std::sync::OnceLock;

use color_eyre::eyre::{eyre, Result};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect, Size},
    style::Style,
    text::{Line, ToLine, ToSpan},
    widgets::{Block, Paragraph},
    Frame,
};
use tokio::sync::{mpsc, Mutex};
use tui_input::backend::crossterm::EventHandler;

use crate::{actions::Action, component::Component, events::Event};

// Global Singleton to request input from the operator
// by providing the prompt to show to the operator
//
// This module "owns" the channel for receiving the input and is given
// the sender for the prompt channel which is "owned" by the UI
//
// channel (OperatorPrompt) - test_to_operator
//         - sender is used by test
//         - recv is used by TUI (owner)
//
// channel (OperatorInput) - operator_to_test
//         - sender is used by tui
//         - revc is used by test (owner)

static OPERATOR_COMMS: OnceLock<Mutex<TestOperatorComms>> = OnceLock::new();

#[derive(Debug)]
pub struct OperatorPrompt(pub String);

#[derive(Debug)]
pub struct OperatorInput(pub String);

struct TestOperatorComms {
    prompt_sender: mpsc::UnboundedSender<OperatorPrompt>,
    operator_recivier: mpsc::UnboundedReceiver<OperatorInput>,
}

pub struct UIOperatorComms {
    pub prompt_receiver: mpsc::UnboundedReceiver<OperatorPrompt>,
    pub operator_sender: mpsc::UnboundedSender<OperatorInput>,
}

const DEFAULT_PROMPT_TEXT: &'static str = "No Input Currently Required";

pub struct Input {
    action_tx: Option<mpsc::UnboundedSender<Action>>,
    ui_comms: UIOperatorComms,
    txt_input: tui_input::Input,
    prompt_text: String,
}

impl Input {
    pub fn new() -> Result<Self> {
        let (input_tx, input_rx) = mpsc::unbounded_channel::<OperatorInput>();
        let (prompt_tx, prompt_rx) = mpsc::unbounded_channel::<OperatorPrompt>();

        OPERATOR_COMMS
            .set(Mutex::new(TestOperatorComms {
                prompt_sender: prompt_tx,
                operator_recivier: input_rx,
            }))
            .map_err(|_| eyre!("Failed to init Operator Comms"))?;

        Ok(Self {
            action_tx: Default::default(),
            ui_comms: UIOperatorComms {
                prompt_receiver: prompt_rx,
                operator_sender: input_tx,
            },
            txt_input: Default::default(),
            prompt_text: DEFAULT_PROMPT_TEXT.to_string(),
        })
    }

    pub fn request(prompt: impl Into<String>) -> Result<String> {
        let mut comms = OPERATOR_COMMS
            .get()
            .expect("Failed to get oncelock")
            .blocking_lock();
        comms.prompt_sender.send(OperatorPrompt(prompt.into()))?;
        let OperatorInput(input) = comms
            .operator_recivier
            .blocking_recv()
            .ok_or(eyre!("Failed to get input"))?;
        Ok(input)
    }

    pub async fn prompt(&mut self) -> Option<Event> {
        self.ui_comms
            .prompt_receiver
            .recv()
            .await
            .map(|p| Event::OperatorPrompt(p.0))
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
    fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let Some(event) = event else {
            return Ok(None);
        };
        match event {
            Event::OperatorInput(e) => Ok(Some(Action::OperatorInput(e))),
            Event::SendInput => Ok(Some(Action::SendInput)),
            Event::OperatorPrompt(p) => Ok(Some(Action::OperatorPrompt(p))),
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
                self.ui_comms.operator_sender.send(input)?;
                Ok(None)
            }
            Action::OperatorPrompt(p) => {
                self.prompt_text = p;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [prompt_area, input_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(3)]).areas(area);

        self.draw_prompt(frame, prompt_area)?;
        self.draw_input(frame, input_area)?;
        Ok(())
    }
}
