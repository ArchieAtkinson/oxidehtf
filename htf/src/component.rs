use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Rect, Size},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{actions::Action, events::Event};

pub trait Component {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        let _ = tx; // to appease clippy
        Ok(())
    }

    fn init(&mut self, area: Size) -> Result<()> {
        let _ = area; // to appease clippy
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let _ = event;
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action; // to appease clippy
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}
