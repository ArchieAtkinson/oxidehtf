use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

use crate::{
    common::*,
    test_runner::{SuiteDataCollectionRaw, TestState},
};

use super::Component;

pub struct WaitingTestDisplay {}

impl WaitingTestDisplay {
    pub fn new() -> Self {
        Self {}
    }

    fn render_waiting_tests(&self, frame: &mut Frame, area: Rect, data: &SuiteDataCollectionRaw) {
        let waiting_tests: Vec<Line> = data.inner[data.current]
            .test_data
            .iter()
            .filter(|test| match test.state {
                TestState::InQueue | TestState::Running(_) => true,
                _ => false,
            })
            .map(|test| Line::from(format!("{} - {}", test.name, test.state)))
            .collect();

        let test_list = Paragraph::new(Text::from(waiting_tests))
            .block(
                Block::bordered()
                    .title("Upcoming Tests")
                    .title_style(Style::default().bold()),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(test_list, area);
    }
}

impl Component for WaitingTestDisplay {
    fn name(&self) -> &str {
        "Test Status Display"
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, data: &SuiteDataCollectionRaw) -> Result<()> {
        self.render_waiting_tests(frame, area, data);
        Ok(())
    }
}
