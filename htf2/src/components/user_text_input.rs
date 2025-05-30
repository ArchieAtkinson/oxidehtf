use color_eyre::eyre::Result;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph},
    Frame,
};
use tokio::sync::mpsc;
use tui_input::backend::crossterm::EventHandler;

use crate::{actions::Action, events::Event, test_runner::TestData, ui::UiAreas};

use super::Component;

const DEFAULT_PROMPT_TEXT: &'static str = "No Input Currently Required";

pub struct UserTextInput {
    action_tx: Option<mpsc::UnboundedSender<Action>>,
    event_tx: Option<mpsc::UnboundedSender<Event>>,
    txt_input: tui_input::Input,
}

impl UserTextInput {
    pub fn new() -> Self {
        Self {
            action_tx: Default::default(),
            event_tx: Default::default(),
            txt_input: Default::default(),
        }
    }

    fn draw_input(&mut self, frame: &mut Frame, area: Rect, data: &TestData) -> Result<()> {
        let current_index = data.current_index;
        let prompt = data[current_index]
            .user_inputs
            .last()
            .map_or(DEFAULT_PROMPT_TEXT.into(), |i| i.prompt.clone());

        // keep 2 for borders and 1 for cursor
        let width = area.width.max(3) - 3;
        let scroll = self.txt_input.visual_scroll(width as usize);
        let input = Paragraph::new(self.txt_input.value())
            .scroll((0, scroll as u16))
            .block(
                Block::bordered()
                    .title(prompt)
                    .title_style(Style::default().bold()),
            );
        frame.render_widget(input, area);

        // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past
        // the end of the input text and one line down from the border to the input line
        let x = self.txt_input.visual_cursor().max(scroll) - scroll + 1;
        frame.set_cursor_position((area.x + x as u16, area.y + 1));
        Ok(())
    }
}

impl Component for UserTextInput {
    fn init(&mut self) -> Result<()> {
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
            Action::TerminalInput(e) => {
                let _ = self.txt_input.handle_event(&e);
            }
            Action::SendInput => {
                return Ok(Some(Action::OperatorTextInput(
                    self.txt_input.value_and_reset(),
                )));
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: &UiAreas, data: &TestData) -> Result<()> {
        assert_eq!(area.operator.height, 3);
        self.draw_input(frame, area.operator, data)?;

        Ok(())
    }
}
