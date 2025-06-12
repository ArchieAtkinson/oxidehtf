use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    Frame, Terminal,
};
use std::io::Stdout;

use crate::common::*;

pub struct UiAreas {
    pub test_progress: Rect,
    pub operator: Rect,
    pub current_test: Rect,
    pub completed_list: Rect,
    pub waiting_list: Rect,
}

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    event_tx: UnboundedSender<Event>,
}

impl Ui {
    pub fn new(event_tx: UnboundedSender<Event>) -> Self {
        Self {
            terminal: ratatui::init(),
            event_tx,
        }
    }

    pub fn start(&self) {
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

    pub fn render<F>(&mut self, draw_callback: F) -> Result<()>
    where
        F: FnOnce(&mut Frame, &UiAreas) -> Result<()>,
    {
        let mut result = Ok(());
        self.terminal.draw(|f| {
            result = {
                let [test_progress, operator, current_test, lists_of_tests] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(10),
                    Constraint::Min(1),
                ])
                .areas(f.area());

                let [completed_list, waiting_list] =
                    Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                        .areas(lists_of_tests);

                let areas = UiAreas {
                    test_progress,
                    operator,
                    current_test,
                    completed_list,
                    waiting_list,
                };

                draw_callback(f, &areas)
            }
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
