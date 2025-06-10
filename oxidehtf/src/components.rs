use crate::{actions::Action, events::Event, test_runner::test_data::TestData, ui::UiAreas};
use color_eyre::Result;
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

pub mod completed_tests;
pub mod current_test;
pub mod suite_progress;
pub mod user_text_input;
pub mod waiting_tests;

pub trait Component {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str;

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

    fn can_focus(&self) -> bool {
        false
    }

    fn focus(&mut self) {
        ()
    }

    fn blur(&mut self) {
        ()
    }

    fn draw(&mut self, frame: &mut Frame, area: &UiAreas, state: &TestData) -> Result<()>;
}
