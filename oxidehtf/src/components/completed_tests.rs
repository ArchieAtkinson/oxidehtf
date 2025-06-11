// use cli_log::*;
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, List},
    Frame,
};

use crate::{
    test_runner::test_data::{SuiteDataInner, TestState},
    ui::UiAreas,
};

use super::Component;

pub struct CompletedTestDisplay {}

impl CompletedTestDisplay {
    pub fn new() -> Self {
        Self {}
    }

    fn render_completed_tests(&self, frame: &mut Frame, area: Rect, data: &SuiteDataInner) {
        let completed_tests = data
            .test_metadata
            .iter()
            .filter(|test| match test.state {
                TestState::Done(_) => true,
                _ => false,
            })
            .rev()
            .map(|test| Line::from(format!("{} - {}", test.name, test.state)));

        let test_list = List::new(completed_tests).block(
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

    fn draw(&mut self, frame: &mut Frame, area: &UiAreas, data: &SuiteDataInner) -> Result<()> {
        self.render_completed_tests(frame, area.completed_list, data);
        Ok(())
    }
}
