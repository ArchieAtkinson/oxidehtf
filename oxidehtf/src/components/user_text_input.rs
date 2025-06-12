use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{common::*, event_handlers::TextInputHandler, test_runner::SuiteDataRaw, ui::UiAreas};

use super::Component;

pub struct UserTextInput {
    action_tx: Option<UnboundedSender<Action>>,
    event_tx: Option<UnboundedSender<Event>>,
    txt_input: tui_input::Input,
    prompt: String,
    is_focused: bool,
}

impl UserTextInput {
    pub fn new() -> Self {
        Self {
            action_tx: Default::default(),
            event_tx: Default::default(),
            txt_input: Default::default(),
            prompt: String::new(),
            is_focused: false,
        }
    }

    fn draw_input(&mut self, frame: &mut Frame, area: Rect, _data: &SuiteDataRaw) -> Result<()> {
        let border_style = if self.is_focused {
            Style::default().yellow()
        } else {
            Style::default()
        };

        // keep 2 for borders and 1 for cursor
        let width = area.width.max(3) - 3;
        let scroll = self.txt_input.visual_scroll(width as usize);
        let input = Paragraph::new(self.txt_input.value())
            .scroll((0, scroll as u16))
            .block(
                Block::bordered()
                    .border_style(border_style)
                    .title(self.prompt.clone())
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

    fn name(&self) -> &str {
        "User Text Input"
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx.clone());
        Ok(())
    }

    fn register_event_handler(&mut self, tx: UnboundedSender<Event>) -> Result<()> {
        self.event_tx = Some(tx.clone());
        Ok(())
    }

    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        if !self.is_focused {
            return Ok(None);
        }

        match event {
            Event::Key(key)
                if key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE =>
            {
                Ok(Some(Action::SendInput))
            }
            _ => Ok(TextInputHandler::handle_events(event)),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::InputRequest(e) => {
                if !self.is_focused {
                    return Ok(None);
                }
                self.txt_input
                    .handle(e)
                    .expect("Failed to handle text input");
            }
            Action::SendInput => {
                let input = self.txt_input.value_and_reset();
                return Ok(Some(Action::UserInputValue(input)));
            }
            Action::UserInputPrompt(v) => self.prompt = v,
            _ => (),
        }
        Ok(None)
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn blur(&mut self) {
        self.is_focused = false;
    }

    fn draw(&mut self, frame: &mut Frame, area: &UiAreas, data: &SuiteDataRaw) -> Result<()> {
        assert_eq!(area.operator.height, 3);
        self.draw_input(frame, area.operator, data)?;

        Ok(())
    }
}
