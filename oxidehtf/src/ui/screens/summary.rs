use std::collections::HashMap;

use super::components::Component;
use super::Screen;
use crate::{app::Id, common::*, test_runner::SuiteDataCollectionRaw};
use ratatui::layout::Alignment;
use ratatui::text::Text;
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

    fn focus_next(&mut self, current_focus: &Id) -> Option<Id> {
        Some(match current_focus {
            Id::WelcomeIntro => Id::WelcomeSuites,
            Id::WelcomeSuites => Id::WelcomeIntro,
            _ => panic!("Can't focus next from unknown ID"),
        })
    }

    fn focus_previous(&mut self, current_focus: &Id) -> Option<Id> {
        Some(match current_focus {
            Id::WelcomeIntro => Id::WelcomeSuites,
            Id::WelcomeSuites => Id::WelcomeIntro,
            _ => panic!("Can't focus next from unknown ID"),
        })
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        _components: &mut std::collections::HashMap<Id, Box<dyn Component>>,
        state: &SuiteDataCollectionRaw,
    ) -> Result<()> {
        let mut text = Vec::new();

        for suite in &state.inner {
            let tests = suite
                .test_data
                .iter()
                .map(|f| format!("{} - {}", f.name, f.state));

            text.push(format!("{}", suite.name));
            text.extend(tests);
            text.push(String::from(""));
        }

        let summary = Text::from_iter(text.clone()).alignment(Alignment::Center);
        let [top_area, summary_area, bottom_area] = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(text.len() as u16),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(frame.area());

        let style = Style::default();

        let padding_top = Block::new().style(style);
        let padding_bottom = Block::new().style(style);

        frame.render_widget(padding_top, top_area);
        frame.render_widget(summary, summary_area);
        frame.render_widget(padding_bottom, bottom_area);
        Ok(())
    }
}
