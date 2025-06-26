use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph},
    Frame,
};
use tokio::sync::oneshot;

use crate::{
    common::*, event_handlers::TextInputHandler, test_runner::SuiteDataCollectionRaw,
    ui::screens::components::Attribute,
};

use super::Component;

pub struct UserTextInput {
    txt_input: tui_input::Input,
    prompt: String,
    is_focused: bool,
    reply: Option<oneshot::Sender<String>>,
}

impl UserTextInput {
    const DEFAULT_PROMPT: &'static str = "No Input Required";

    pub fn new() -> Self {
        Self {
            txt_input: Default::default(),
            prompt: Self::DEFAULT_PROMPT.into(),
            is_focused: false,
            reply: None,
        }
    }

    fn draw_input(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        _data: &SuiteDataCollectionRaw,
    ) -> Result<()> {
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
    fn name(&self) -> &str {
        "User Text Input"
    }

    fn handle_event(&mut self, event: &Event) -> Result<Option<Action>> {
        match event {
            Event::Key(key)
                if key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE =>
            {
                Ok(Some(Action::SendInput))
            }
            _ => Ok(TextInputHandler::handle_events(event)),
        }
    }

    fn update(&mut self, action: &mut Action) -> Result<Option<Action>> {
        match action {
            Action::UserKeyInputRequest(e) => {
                if !self.is_focused {
                    return Ok(None);
                }
                self.txt_input.handle(e.clone());
            }
            Action::SendInput => {
                let input = self.txt_input.value_and_reset();
                self.prompt = Self::DEFAULT_PROMPT.into();
                let sender = self.reply.take().unwrap();
                sender.send(input).unwrap();
                info!("Sent!");
                return Ok(None);
            }
            Action::UserInputPrompt(v, ref mut c) => {
                self.prompt = v.clone();
                self.reply = c.take();
            }
            _ => (),
        }
        Ok(None)
    }

    fn set_attr(&mut self, attr: Attribute) -> Result<()> {
        match attr {
            Attribute::Focus(b) => {
                self.is_focused = b.unwrap();
                Ok(())
            }
            _ => Err(eyre!("Unknown Attr in {}", self.name())),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, data: &SuiteDataCollectionRaw) -> Result<()> {
        self.draw_input(frame, area, data)?;

        Ok(())
    }
}
