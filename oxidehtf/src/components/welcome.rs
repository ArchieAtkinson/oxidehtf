use crate::{app::Screen, common::*};
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{test_runner::SuiteDataRaw, ui::UiAreas};

use super::Component;

pub struct WeclomeDisplay {
    event_tx: Option<UnboundedSender<Event>>,
    is_focused: bool,
}

impl WeclomeDisplay {
    pub fn new() -> Self {
        Self {
            event_tx: None,
            is_focused: false,
        }
    }

    fn render_progress(&self, frame: &mut Frame, area: Rect, _data: &SuiteDataRaw) {
        let text = vec![
            "Welcome to OxideHTF!".into(),
            "Press Tab to change focus, and Esc to quit.".into(),
            "Press any other key to start.".into(),
        ];

        let [top_area, centre_area, bottom_area] = Layout::vertical([
            Constraint::Min(1),
            Constraint::Max(text.len() as u16),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(area);

        let style = Style::default();

        let padding_top = Block::new().style(style);
        let padding_bottom = Block::new().style(style);

        let welcome_text = Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center);

        frame.render_widget(padding_top, top_area);
        frame.render_widget(welcome_text, centre_area);
        frame.render_widget(padding_bottom, bottom_area);
    }
}

impl Component for WeclomeDisplay {
    fn name(&self) -> &str {
        "Test Suite Progress Display"
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
