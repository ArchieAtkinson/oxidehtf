// use cli_log::*;
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Gauge, List, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc;

use crate::{
    actions::Action,
    events::Event,
    test_runner::test_data::{TestData, TestState},
    ui::UiAreas,
};

use super::Component;

pub struct TestStatusDisplay {
    action_tx: Option<mpsc::UnboundedSender<Action>>,
    event_tx: Option<mpsc::UnboundedSender<Event>>,
}

impl TestStatusDisplay {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            event_tx: None,
        }
    }

    fn render_progress(&self, frame: &mut Frame, area: Rect, data: &TestData) {
        let tests_finished = data
            .iter()
            .filter(|test| match test.state {
                TestState::Done(_) => true,
                _ => false,
            })
            .count() as f64;

        let total_tests = data.len() as f64;

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

    fn render_waiting_tests(&self, frame: &mut Frame, area: Rect, data: &TestData) {
        let waiting_tests: Vec<Line> = data
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

    fn render_completed_tests(&self, frame: &mut Frame, area: Rect, data: &TestData) {
        let completed_tests = data
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

impl Component for TestStatusDisplay {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Test Status Display"
    }

    fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx.clone());
        Ok(())
    }

    fn register_event_handler(&mut self, tx: mpsc::UnboundedSender<Event>) -> Result<()> {
        self.event_tx = Some(tx.clone());
        Ok(())
    }

    fn handle_events(&mut self, event: crate::events::Event) -> Result<Option<Action>> {
        match event {
            _ => (),
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: &UiAreas, data: &TestData) -> Result<()> {
        assert_eq!(area.test_progress.height, 1);

        self.render_progress(frame, area.test_progress, data);

        let [completed_area, waiting_area] =
            Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                .areas(area.test_display);

        self.render_completed_tests(frame, completed_area, data);
        self.render_waiting_tests(frame, waiting_area, data);

        Ok(())
    }
}
