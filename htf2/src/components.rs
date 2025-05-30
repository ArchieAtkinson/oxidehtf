use crate::{actions::Action, events::Event, test_runner::TestData, ui::UiAreas};
use color_eyre::Result;
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) mod test_status;
pub(crate) mod user_text_input;

pub trait Component {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        let _ = tx;
        Ok(())
    }

    fn register_event_handler(&mut self, tx: UnboundedSender<Event>) -> Result<()> {
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

    fn draw(&mut self, frame: &mut Frame, area: &UiAreas, state: &TestData) -> Result<()>;
}
