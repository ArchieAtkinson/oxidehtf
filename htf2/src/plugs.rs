use color_eyre::Result;
use tokio::sync::mpsc::UnboundedSender;

use crate::events::Event;

pub(crate) mod user_text_input;

pub trait Plug {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn register_event_handler(&mut self, tx: UnboundedSender<Event>) -> Result<()> {
        let _ = tx;
        Ok(())
    }
}
