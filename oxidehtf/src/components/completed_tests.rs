use ratatui::{
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{
    common::*,
    test_runner::{SuiteDataCollectionRaw, TestState},
    ui::UiAreas,
};

use super::Component;

pub struct CompletedTestDisplay {}

impl CompletedTestDisplay {
    pub fn new() -> Self {
        Self {}
    }

    fn render_completed_tests(&self, frame: &mut Frame, area: Rect, data: &SuiteDataCollectionRaw) {
        let completed_tests = data
            .current_suite()
            .test_data
            .iter()
            .filter(|test| match test.state {
                TestState::Done(_) => true,
                _ => false,
            })
            .rev()
            .map(|test| format!("{} - {}", test.name, test.state));

        let test_list = Paragraph::new(Text::from_iter(completed_tests)).block(
            Block::bordered()
                .title("Completed Tests")
                .title_style(Style::default().bold()),
        );

        frame.render_widget(test_list, area);
    }
}

impl Component for CompletedTestDisplay {
    fn name(&self) -> &str {
        "Test Status Display"
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        area: &UiAreas,
        data: &SuiteDataCollectionRaw,
    ) -> Result<()> {
        self.render_completed_tests(frame, area.completed_list, data);
        Ok(())
    }
}
