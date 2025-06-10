// use cli_log::info;
// use cli_log::*;
use color_eyre::Result;
use ratatui::layout::Margin;
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Row, Table, TableState},
    Frame,
};
use tokio::sync::mpsc;

use crate::{
    actions::Action,
    event_handlers::movement_handler::MovementHandler,
    events::Event,
    test_runner::test_data::{TestData, TestState},
    ui::UiAreas,
};

use super::Component;

enum Scroll {
    Up,
    Down,
}

pub struct CurrentTestDisplay {
    action_tx: Option<mpsc::UnboundedSender<Action>>,
    event_tx: Option<mpsc::UnboundedSender<Event>>,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    is_focused: bool,
    current_rows_seen: usize,
    total_measurements: usize,
}

impl CurrentTestDisplay {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            event_tx: None,
            table_state: TableState::default(),
            is_focused: false,
            current_rows_seen: 0,
            total_measurements: 0,
            scrollbar_state: ScrollbarState::new(0),
        }
    }

    fn scroll(&mut self, direction: Scroll) {
        let offset = self.table_state.offset_mut();

        *offset = match direction {
            Scroll::Down => {
                self.scrollbar_state.next();

                offset.saturating_add(1)
            }
            Scroll::Up => {
                self.scrollbar_state.prev();
                offset.saturating_sub(1)
            }
        };

        let max_offset = self.total_measurements - self.current_rows_seen;
        *offset = (*offset).clamp(0 as usize, max_offset);
    }

    fn render_current_test(&mut self, frame: &mut Frame, area: Rect, data: &TestData) {
        let current_test = data.iter().find(|t| match t.state {
            TestState::Running(_) => true,
            _ => false,
        });

        // 2 for border, 1 for header = 3
        self.current_rows_seen = usize::from(area.height) - 3;

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

        self.total_measurements = rows.len();

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

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        self.scrollbar_state = self
            .scrollbar_state
            .content_length(self.total_measurements)
            .viewport_content_length(self.current_rows_seen);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.scrollbar_state,
        )
    }
}

impl Component for CurrentTestDisplay {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Current Test Measurements"
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
        if self.is_focused {
            Ok(MovementHandler::handle_event(event))
        } else {
            Ok(None)
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::MoveUp => self.scroll(Scroll::Up),
            Action::MoveDown => self.scroll(Scroll::Down),
            _ => (),
        }
        Ok(None)
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
        let area = area.current_test;
        self.render_current_test(frame, area, data);
        self.render_scrollbar(frame, area);
        Ok(())
    }
}
