// use cli_log::*;
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Gauge, List},
    Frame,
};
use tokio::sync::mpsc;

use crate::{
    actions::Action,
    events::Event,
    test_runner::{TestRunnerState, TestState},
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

    fn render_progress(&self, frame: &mut Frame, area: Rect, state: &TestRunnerState) {
        let tests_finished = state
            .tests
            .iter()
            .filter(|test| {
                test.data.state == TestState::Passed || test.data.state == TestState::Failed
            })
            .count() as f64;

        let total_tests = state.tests.len() as f64;

        let mut progress_ratio: f64 = tests_finished / total_tests;
        if total_tests == 0.0 {
            progress_ratio = 0.0;
        }

        let progress_percentage = (progress_ratio * 100.0) as i32;
        let bar = Gauge::default()
            .gauge_style(Style::new().white().on_black().bold())
            .label(format!("Test Suite Progress: {}%", progress_percentage))
            .ratio(progress_ratio);

        frame.render_widget(bar, area);
    }

    fn render_waiting_tests(&self, frame: &mut Frame, area: Rect, state: &TestRunnerState) {
        let waiting_tests = state
            .tests
            .iter()
            .filter(|test| {
                test.data.state == TestState::Waiting || test.data.state == TestState::Running
            })
            .map(|test| format!("{} {:?}", test.data.name, test.data.state));

        let test_list = List::new(waiting_tests).block(
            Block::bordered()
                .title("Upcoming Tests")
                .title_style(Style::default().bold()),
        );

        frame.render_widget(test_list, area);
    }

    fn render_completed_tests(&self, frame: &mut Frame, area: Rect, state: &TestRunnerState) {
        let completed_tests = state
            .tests
            .iter()
            .filter(|test| {
                test.data.state == TestState::Passed || test.data.state == TestState::Failed
            })
            .rev()
            .map(|test| format!("{} {:?}", test.data.name, test.data.state));

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
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: &UiAreas, state: &TestRunnerState) -> Result<()> {
        assert_eq!(area.test_progress.height, 1);

        self.render_progress(frame, area.test_progress, state);

        let [completed_area, waiting_area] =
            Layout::horizontal([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)])
                .areas(area.test_list);

        self.render_completed_tests(frame, completed_area, state);
        self.render_waiting_tests(frame, waiting_area, state);

        Ok(())
    }
}
