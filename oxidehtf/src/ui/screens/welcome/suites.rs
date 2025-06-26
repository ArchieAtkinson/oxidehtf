use ratatui::layout::Margin;
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Row, Table, TableState},
    Frame,
};

use crate::ui::screens::components::Attribute;
use crate::{common::*, event_handlers::MovementHandler, test_runner::SuiteDataCollectionRaw};

use super::Component;

enum Scroll {
    Up,
    Down,
}

pub struct SuitesDisplay {
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    is_focused: bool,
    current_rows_seen: usize,
    total_suites: usize,
}

impl SuitesDisplay {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            is_focused: false,
            current_rows_seen: 0,
            total_suites: 0,
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

        let max_offset = self.total_suites - self.current_rows_seen;
        *offset = (*offset).clamp(0 as usize, max_offset);
    }

    fn render_current_test(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        data: &SuiteDataCollectionRaw,
    ) {
        // 2 for border, 1 for header = 3
        self.current_rows_seen = usize::from(area.height) - 3;
        let mut rows = Vec::new();

        for (index, suite) in data.inner.iter().enumerate() {
            let name = suite.name.to_string();
            let priority = suite.priority.to_string();
            let position = (index + 1).to_string();
            let row = vec![position, name, priority];
            rows.push(Row::from_iter(row));
        }

        self.total_suites = data.inner.len();

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
            .block(Block::bordered().border_style(border_style))
            .header(
                Row::new(vec!["Run Order", "Name", "Priority"]).style(Style::new().underlined()),
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
            .content_length(self.total_suites)
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

impl Component for SuitesDisplay {
    fn name(&self) -> &str {
        "Suites"
    }

    fn handle_event(&mut self, event: &Event) -> Result<Option<Action>> {
        if self.is_focused {
            Ok(MovementHandler::handle_event(event))
        } else {
            Ok(None)
        }
    }

    fn update(&mut self, action: &mut Action) -> Result<Option<Action>> {
        match action {
            Action::MoveUp => self.scroll(Scroll::Up),
            Action::MoveDown => self.scroll(Scroll::Down),
            _ => (),
        }
        Ok(None)
    }

    fn set_attr(&mut self, attr: Attribute) -> Result<()> {
        match attr {
            Attribute::Focus(b) => {
                self.is_focused = b.unwrap();
                Ok(())
            }
            _ => Err(eyre!("Unknown Attr in {}", self.name())),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, data: &SuiteDataCollectionRaw) -> Result<()> {
        let area = area.inner(Margin::new(area.width / 8, 1));
        self.render_current_test(frame, area, data);
        self.render_scrollbar(frame, area);
        Ok(())
    }
}
