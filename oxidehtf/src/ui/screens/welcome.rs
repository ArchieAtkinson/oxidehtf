use super::components::{Attribute, Component};
use super::Screen;
use crate::{app::Id, common::*, test_runner::SuiteDataCollectionRaw};
use intro::IntroDisplay;
use ratatui::{
    layout::{Constraint, Flex, Layout},
    style::Style,
    widgets::Block,
    Frame,
};
use suites::SuitesDisplay;

pub mod intro;
pub mod suites;

pub struct WelcomeScreen {
    event_tx: UnboundedSender<Event>,
}

impl WelcomeScreen {
    pub fn new(event_tx: UnboundedSender<Event>) -> Self {
        Self { event_tx }
    }
}

impl Screen for WelcomeScreen {
    fn name(&self) -> &str {
        "Welcome"
    }

    fn activate(
        &mut self,
        components: &mut std::collections::HashMap<crate::app::Id, Box<dyn Component>>,
    ) -> Option<Id> {
        components.insert(Id::WelcomeIntro, Box::new(IntroDisplay::new()));
        components.insert(Id::WelcomeSuites, Box::new(SuitesDisplay::new()));

        Some(Id::WelcomeIntro)
    }

    fn deactivate(&mut self, components: &mut std::collections::HashMap<Id, Box<dyn Component>>) {
        components.remove(&Id::WelcomeIntro);
        components.remove(&Id::WelcomeSuites);
    }

    fn focus_next(&mut self, current_focus: Id) -> Id {
        match current_focus {
            Id::WelcomeIntro => Id::WelcomeSuites,
            Id::WelcomeSuites => Id::WelcomeIntro,
            _ => panic!("Can't focus next from unknown ID"),
        }
    }

    fn focus_previous(&mut self, current_focus: Id) -> Id {
        match current_focus {
            Id::WelcomeIntro => Id::WelcomeSuites,
            Id::WelcomeSuites => Id::WelcomeIntro,
            _ => panic!("Can't focus next from unknown ID"),
        }
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        components: &mut std::collections::HashMap<Id, Box<dyn Component>>,
        state: &SuiteDataCollectionRaw,
    ) -> Result<()> {
        let intro_constraint = match components
            .get(&Id::WelcomeIntro)
            .unwrap()
            .get_attr(Attribute::VertConstraint(None))?
        {
            Attribute::VertConstraint(mut c) if c.is_some() => c.take().unwrap(),
            _ => return Err(eyre!("Bad Attr")),
        };

        let [top_area, intro_area, suites_area, bottom_area] = Layout::vertical([
            Constraint::Min(1),
            intro_constraint,
            Constraint::Min(1),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(frame.area());

        let style = Style::default();

        let padding_top = Block::new().style(style);
        let padding_bottom = Block::new().style(style);

        frame.render_widget(padding_top, top_area);
        components
            .get_mut(&Id::WelcomeIntro)
            .unwrap()
            .draw(frame, intro_area, state)?;
        components
            .get_mut(&Id::WelcomeSuites)
            .unwrap()
            .draw(frame, suites_area, state)?;
        frame.render_widget(padding_bottom, bottom_area);
        Ok(())
    }
}
