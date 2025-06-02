use color_eyre::Result;
use tokio::sync::mpsc::UnboundedSender;

use crate::events::{Event, PlugEvent};

pub(crate) mod user_text_input;

pub struct PlugEventSender {
    tx: UnboundedSender<Event>,
}

impl PlugEventSender {
    pub(crate) fn new(tx: UnboundedSender<Event>) -> Self {
        Self { tx }
    }

    pub fn send(&self, event: PlugEvent) -> Result<()> {
        self.tx.send(Event::PlugEvent(event))?;
        Ok(())
    }
}

pub trait Plug: Default {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn request_sender(&mut self, sender: PlugEventSender) {
        let _ = sender;
    }

    fn teardown(&mut self) -> Result<()> {
        Ok(())
    }
}
