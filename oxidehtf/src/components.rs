use crate::{common::*, test_runner::SuiteDataCollectionRaw, ui::UiAreas};
use ratatui::Frame;

pub mod completed_tests;
pub mod current_test;
pub mod suite_progress;
pub mod user_text_input;
pub mod waiting_tests;
pub mod welcome;

pub use completed_tests::CompletedTestDisplay;
pub use current_test::CurrentTestDisplay;
pub use suite_progress::SuiteProgressDisplay;
pub use user_text_input::UserTextInput;
pub use waiting_tests::WaitingTestDisplay;
pub use welcome::WeclomeDisplay;

pub trait Component {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    #[allow(dead_code)]
    fn name(&self) -> &str;

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

    fn draw(
        &mut self,
        frame: &mut Frame,
        area: &UiAreas,
        state: &SuiteDataCollectionRaw,
    ) -> Result<()>;
}
