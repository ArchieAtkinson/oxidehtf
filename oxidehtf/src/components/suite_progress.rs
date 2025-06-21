// use cli_log::*;
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Gauge,
    Frame,
};

use crate::test_runner::{SuiteDataCollectionRaw, TestState};
use crate::ui::UiAreas;

use super::Component;

pub struct SuiteProgressDisplay {}

impl SuiteProgressDisplay {
    pub fn new() -> Self {
        Self {}
    }

    fn render_progress(&self, frame: &mut Frame, area: Rect, data: &SuiteDataCollectionRaw) {
        let tests_finished = data.inner[data.current]
            .test_data
            .iter()
            .filter(|test| match test.state {
                TestState::Done(_) => true,
                _ => false,
            })
            .count() as f64;

        let total_tests = data.inner[data.current].test_data.len() as f64;

        let mut progress_ratio: f64 = tests_finished / total_tests;
        if total_tests == 0.0 {
            progress_ratio = 0.0;
        }

        let progress_percentage = (progress_ratio * 100.0) as i32;
        let bar = Gauge::default()
            .gauge_style(Style::new().black().on_white().bold())
            .label(format!(
                "Test Suite Progress: {}% ({}/{})",
                progress_percentage, tests_finished as i32, total_tests as i32
            ))
            .white()
            .ratio(progress_ratio);

        frame.render_widget(bar, area);
    }
}

impl Component for SuiteProgressDisplay {
    fn name(&self) -> &str {
        "Test Suite Progress Display"
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        area: &UiAreas,
        data: &SuiteDataCollectionRaw,
    ) -> Result<()> {
        assert_eq!(area.test_progress.height, 1);
        self.render_progress(frame, area.test_progress, data);
        Ok(())
    }
}
