use color_eyre::Result;
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

use crate::{actions::Action, events::Event, ui::UiArea};

pub(crate) mod operator;
pub(crate) mod test_runner;

pub trait Component {
    fn init(&mut self) {}
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        let _ = tx;
        Ok(())
    }

    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        let _ = event;
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action;
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: &UiArea) -> Result<()>;
}
