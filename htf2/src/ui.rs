use color_eyre::eyre::Result;
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    Frame, Terminal,
};
use std::io::Stdout;
use tokio::sync::mpsc;

use crate::events::Event;

pub struct UiAreas {
    pub test_progress: Rect,
    pub operator: Rect,
    pub test_display: Rect,
}

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl Ui {
    pub fn new(event_tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            terminal: ratatui::init(),
            event_tx,
        }
    }

    pub fn start(&self) {
        let event_loop = Self::event_loop(self.event_tx.clone());
        tokio::spawn(async { event_loop.await });
    }

    pub async fn event_loop(event_tx: mpsc::UnboundedSender<Event>) {
        loop {
            let mut stream = EventStream::new();

            let Some(Ok(event)) = stream.next().await else {
                break;
            };

            let _ = event_tx.send(Event::CrosstermEvent(event));
        }
    }

    pub fn render<F>(&mut self, draw_callback: F) -> Result<()>
    where
        F: FnOnce(&mut Frame, &UiAreas) -> Result<()>,
    {
        let mut result = Ok(());
        self.terminal.draw(|f| {
            result = {
                let [test_progress, operator, test_list] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .areas(f.area());

                let areas = UiAreas {
                    test_progress,
                    operator,
                    test_display: test_list,
                };

                draw_callback(f, &areas)
            }
        })?;

        result
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
