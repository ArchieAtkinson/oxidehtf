use crate::{
    common::*,
    test_runner::SuiteDataCollectionRaw,
    ui::{
        screens::components::{Attribute, Component},
        Screens,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    text::Text,
    Frame,
};

pub struct IntroDisplay {
    is_focused: bool,
    text: Vec<&'static str>,
}

impl IntroDisplay {
    pub fn new() -> Self {
        let text = vec![
            "Welcome to OxideHTF!",
            "Press Tab to change focus, and Esc to quit.",
            "Press any other key to start.",
        ];
        Self {
            is_focused: false,
            text,
        }
    }

    fn welcome_screen(&self, frame: &mut Frame, area: Rect, _data: &SuiteDataCollectionRaw) {
        let welcome_text = Text::from_iter(self.text.clone()).alignment(Alignment::Center);
        frame.render_widget(welcome_text, area);
    }
}

impl Component for IntroDisplay {
    fn name(&self) -> &str {
        "Test Suite Progress Display"
    }

    fn handle_event(&mut self, event: &Event) -> Result<Option<Action>> {
        match event {
            Event::Key(_) => Ok(Some(Action::ChangeScreen(Screens::RunningTests))),
            _ => Ok(None),
        }
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

    fn get_attr(&self, attr: Attribute) -> Result<Attribute> {
        match attr {
            Attribute::VertConstraint(_) => Ok(Attribute::VertConstraint(Some(
                Constraint::Length(self.text.len() as u16),
            ))),
            _ => Err(eyre!("Unknown Attr in {}", self.name())),
        }
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        _data: &SuiteDataCollectionRaw,
    ) -> Result<()> {
        self.welcome_screen(frame, area, _data);
        Ok(())
    }
}
