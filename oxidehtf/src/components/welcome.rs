use crate::{app::Screen, common::*};
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{test_runner::SuiteDataRaw, ui::UiAreas};

use super::Component;

pub struct WeclomeDisplay {
    action_tx: Option<UnboundedSender<Action>>,
    event_tx: Option<UnboundedSender<Event>>,
    is_focused: bool,
}

impl WeclomeDisplay {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            event_tx: None,
            is_focused: false,
        }
    }

    fn render_progress(&self, frame: &mut Frame, area: Rect, _data: &SuiteDataRaw) {
        let text = vec![
            "Welcome to OxideHTF!".into(),
            "Press Tab to change focus, and Esc to quit".into(),
            "Press any other key to start".into(),
        ];
        let welcome_text = Paragraph::new(text)
            .block(Block::bordered())
            .style(Style::new().black().on_white())
            .alignment(Alignment::Center);
        frame.render_widget(welcome_text, area);
    }
}

impl Component for WeclomeDisplay {
    fn name(&self) -> &str {
        "Test Suite Progress Display"
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
            Event::Key(_) => Ok(Some(Action::ChangeScreen(Screen::RunningTests))),
            _ => Ok(None),
        }
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

    fn draw(&mut self, frame: &mut Frame, _area: &UiAreas, _data: &SuiteDataRaw) -> Result<()> {
        self.render_progress(frame, frame.area(), _data);
        Ok(())
    }
}
