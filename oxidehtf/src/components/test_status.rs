// use cli_log::*;
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Gauge, List, Paragraph, Row, Table, TableState, Wrap},
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
    current_test_table_state: TableState,
    is_focused: bool,
}

impl TestStatusDisplay {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            event_tx: None,
            current_test_table_state: TableState::default(),
            is_focused: false,
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

    fn render_current_test(&mut self, frame: &mut Frame, area: Rect, data: &TestData) {
        let current_test = data.iter().find(|t| match t.state {
            TestState::Running(_) => true,
            _ => false,
        });

        let current_test_name: String = {
            if let Some(current_test) = current_test {
                current_test.name.into()
            } else {
                "No Running Test".into()
            }
        };

        let dut: String = {
            if data.dut_id.is_empty() {
                "DUT not set".into()
            } else {
                data.dut_id.clone()
            }
        };

        let mut rows = Vec::new();

        if let Some(current_test) = current_test {
            for data in &current_test.user_data {
                let name = data.0.clone();
                let Some(value) = data.1.value.clone() else {
                    break;
                };
                let value = format!("{}", value);
                let unit = data.1.unit.clone().unwrap_or("None".into());
                let row = Row::new(vec![name.into(), value, unit.into()]);
                rows.push(row);
            }
        }

        let rows = rows.iter_mut().enumerate().map(|(i, r)| {
            if i % 2 == 0 {
                r.clone().black().on_gray()
            } else {
                r.clone()
            }
        });

        let border_style = if self.is_focused {
            Style::default().yellow()
        } else {
            Style::default()
        };

        // Columns widths are constrained in the same way as Layout...
        let widths = [Constraint::Min(5), Constraint::Min(5), Constraint::Min(5)];
        let table = Table::new(rows, widths)
            .block(
                Block::bordered()
                    .border_style(border_style)
                    .title(format!(
                        " DUT: {} - Current Test: {} ",
                        dut, current_test_name,
                    ))
                    .title_style(Style::default().bold()),
            )
            .header(
                Row::new(vec!["Measurement Name", "Value", "Units"])
                    .style(Style::new().underlined()),
            )
            .highlight_symbol(">>");
        frame.render_stateful_widget(table, area, &mut self.current_test_table_state);
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
            Event::CrosstermEvent(crossterm_event) => {
                if self.is_focused {
                    if let crossterm::event::Event::Key(key) = crossterm_event {
                        match key.code {
                            KeyCode::Char('k') => self.current_test_table_state.select_previous(),
                            KeyCode::Char('j') => self.current_test_table_state.select_next(),
                            _ => (),
                        }
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
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

    fn draw(&mut self, frame: &mut Frame, area: &UiAreas, data: &TestData) -> Result<()> {
        assert_eq!(area.test_progress.height, 1);

        self.render_progress(frame, area.test_progress, data);

        let [current_test, list_tests] =
            Layout::vertical([Constraint::Length(10), Constraint::Min(1)]).areas(area.test_display);

        self.render_current_test(frame, current_test, data);

        let [completed_area, waiting_area] =
            Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                .areas(list_tests);

        self.render_completed_tests(frame, completed_area, data);
        self.render_waiting_tests(frame, waiting_area, data);

        Ok(())
    }
}
