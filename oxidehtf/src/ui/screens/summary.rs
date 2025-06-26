use std::collections::HashMap;

use super::components::Component;
use super::Screen;
use crate::test_runner::{TestDone, TestState};
use crate::{app::Id, common::*, test_runner::SuiteDataCollectionRaw};
use color_eyre::owo_colors::OwoColorize;
use ratatui::layout::Alignment;
use ratatui::style::{Modifier, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::{
    layout::{Constraint, Flex, Layout},
    style::Style,
    widgets::Block,
    Frame,
};

pub struct SummaryScreen {}

impl SummaryScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for SummaryScreen {
    fn name(&self) -> &str {
        "Welcome"
    }

    fn activate(
        &mut self,
        _components: &mut HashMap<crate::app::Id, Box<dyn Component>>,
    ) -> Option<Id> {
        None
    }

    fn deactivate(&mut self, _components: &mut HashMap<Id, Box<dyn Component>>) {}

    fn focus_next(&mut self, _current_focus: &Id) -> Option<Id> {
        None
    }

    fn focus_previous(&mut self, _current_focus: &Id) -> Option<Id> {
        None
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        _components: &mut std::collections::HashMap<Id, Box<dyn Component>>,
        state: &SuiteDataCollectionRaw,
    ) -> Result<()> {
        let mut text = Text::default().centered();

        let title = Line::from(Span::styled("Summary", Style::default().bold()));
        text.push_line(title);
        text.push_line("");

        for suite in &state.inner {
            let suite_name = Span::styled(format!("{}", suite.name), Style::default().underlined());

            let tests = suite.test_data.iter().map(|f| {
                let style = match f.state {
                    TestState::Done(ref d) => match d {
                        TestDone::Failed(_) => Style::default().red(),
                        TestDone::Passed => Style::default().green(),
                    },
                    _ => panic!(""),
                };

                let line = Line::from(vec![
                    Span::from(f.name),
                    Span::raw(" - "),
                    Span::styled(format!("{}", f.state), style),
                ]);
                line
            });

            text.push_line(suite_name);
            text.extend(tests);
            text.push_line("");
        }

        let [top_area, summary_area, bottom_area] = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(text.height() as u16),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(frame.area());

        let style = Style::default();

        let padding_top = Block::new().style(style);
        let padding_bottom = Block::new().style(style);

        frame.render_widget(padding_top, top_area);
        frame.render_widget(text, summary_area);
        frame.render_widget(padding_bottom, bottom_area);
        Ok(())
    }
}
