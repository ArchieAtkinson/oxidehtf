use color_eyre::Result;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::{actions::Action, events::Event};

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

    fn draw(&mut self, frame: &mut Frame, area: &[Rect]) -> Result<()>;
}
