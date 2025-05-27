use color_eyre::eyre::Result;
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::io::Stdout;

use crate::events::Event;

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            terminal: ratatui::init(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        ratatui::restore();
        Ok(())
    }

    async fn next_event(&mut self) -> Result<Option<Event>> {
        let mut events = EventStream::new();
        Ok(events
            .next()
            .await
            .transpose()?
            .map(|e| Event::CrosstermEvent(e)))
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
