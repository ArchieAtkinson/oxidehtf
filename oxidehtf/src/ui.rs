use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{prelude::CrosstermBackend, Terminal};
use screens::{
    components::{Attribute, Component},
    running::RunningScreen,
    welcome::WelcomeScreen,
    Screen,
};
use std::{collections::HashMap, io::Stdout};

use crate::{app::Id, common::*, test_runner::SuiteDataCollectionRaw};

pub mod screens;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screens {
    #[default]
    Welcome,
    RunningTests,
}

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    event_tx: UnboundedSender<Event>,
    focus_stack: Vec<Id>,
    components: HashMap<Id, Box<dyn Component>>,
    screens: HashMap<Screens, Box<dyn Screen>>,
    current_focus: Option<Id>,
    current_screen: Screens,
}

impl Ui {
    pub fn new(event_tx: UnboundedSender<Event>) -> Self {
        Self {
            terminal: ratatui::init(),
            event_tx: event_tx.clone(),

            focus_stack: Vec::new(),
            components: HashMap::new(),
            screens: HashMap::from([
                (
                    Screens::Welcome,
                    Box::new(WelcomeScreen::new(event_tx.clone())) as Box<dyn Screen>,
                ),
                (
                    Screens::RunningTests,
                    Box::new(RunningScreen::new(event_tx.clone())) as Box<dyn Screen>,
                ),
            ]),
            current_focus: Some(Id::WelcomeIntro),
            current_screen: Screens::Welcome,
        }
    }

    pub fn start(&mut self) {
        self.current_focus = self
            .screens
            .get_mut(&self.current_screen)
            .unwrap()
            .activate(&mut self.components);

        for (_, component) in self.components.iter_mut() {
            component
                .register_event_handler(self.event_tx.clone())
                .unwrap();
        }

        self.components
            .get_mut(&self.current_focus.clone().unwrap())
            .unwrap()
            .set_attr(Attribute::Focus(Some(true)))
            .unwrap();

        self.event_tx.send(Event::NOP).unwrap();

        let event_loop = Self::event_loop(self.event_tx.clone());
        tokio::spawn(async { event_loop.await });
    }

    pub async fn event_loop(event_tx: UnboundedSender<Event>) {
        loop {
            let mut stream = EventStream::new();

            let Some(Ok(event)) = stream.next().await else {
                break;
            };

            if let Some(event) = Self::from_crossterm(event) {
                event_tx.send(event).expect("Event stream empty");
            }
        }
    }

    pub fn active(&mut self, screen: Screens) {
        self.components
            .get_mut(&self.current_focus.clone().unwrap())
            .unwrap()
            .set_attr(Attribute::Focus(Some(false)))
            .unwrap();

        self.screens
            .get_mut(&self.current_screen)
            .unwrap()
            .deactivate(&mut self.components);

        self.focus_stack.clear();

        self.current_focus = self
            .screens
            .get_mut(&screen)
            .unwrap()
            .activate(&mut self.components);

        for (_, component) in self.components.iter_mut() {
            component
                .register_event_handler(self.event_tx.clone())
                .unwrap();
        }

        self.components
            .get_mut(&self.current_focus.clone().unwrap())
            .unwrap()
            .set_attr(Attribute::Focus(Some(true)))
            .unwrap();

        self.current_screen = screen;
    }

    pub fn focus_next(&mut self) {
        self.components
            .get_mut(&self.current_focus.clone().unwrap())
            .unwrap()
            .set_attr(Attribute::Focus(Some(false)))
            .unwrap();

        self.current_focus = Some(
            self.screens
                .get_mut(&self.current_screen)
                .unwrap()
                .focus_next(self.current_focus.clone().unwrap()),
        );

        self.components
            .get_mut(&self.current_focus.clone().unwrap())
            .unwrap()
            .set_attr(Attribute::Focus(Some(true)))
            .unwrap();
    }

    pub fn focus_previous(&mut self) {
        self.components
            .get_mut(&self.current_focus.clone().unwrap())
            .unwrap()
            .set_attr(Attribute::Focus(Some(false)))
            .unwrap();

        self.current_focus = Some(
            self.screens
                .get_mut(&self.current_screen)
                .unwrap()
                .focus_previous(self.current_focus.clone().unwrap()),
        );

        self.components
            .get_mut(&self.current_focus.clone().unwrap())
            .unwrap()
            .set_attr(Attribute::Focus(Some(true)))
            .unwrap();
    }

    pub fn focused_component(&mut self) -> Option<&mut Box<dyn Component>> {
        self.components
            .get_mut(&self.current_focus.clone().unwrap())
    }

    pub fn render(&mut self, mut data: SuiteDataCollectionRaw) -> Result<()> {
        let mut result = Ok(());
        self.terminal.draw(|f| {
            result = self.screens.get_mut(&self.current_screen).unwrap().draw(
                f,
                &mut self.components,
                &mut data,
            )
        })?;

        result
    }

    fn from_crossterm(crossterm_event: crossterm::event::Event) -> Option<Event> {
        use crossterm::event::Event as CrosstermEvent;
        match crossterm_event {
            CrosstermEvent::Key(key_event) => Some(Event::Key(key_event)),
            CrosstermEvent::Mouse(mouse_event) => Some(Event::Mouse(mouse_event)),
            CrosstermEvent::Paste(string) => Some(Event::Paste(string)),
            _ => None,
        }
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
