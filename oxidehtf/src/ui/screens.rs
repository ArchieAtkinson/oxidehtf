use std::collections::HashMap;

use crate::{app::Id, common::*, test_runner::SuiteDataCollectionRaw};
use components::Component;
use ratatui::Frame;

pub mod components;
pub mod running;
pub mod summary;
pub mod welcome;

pub trait Screen {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    fn activate(&mut self, components: &mut HashMap<Id, Box<dyn Component>>) -> Option<Id>;
    fn deactivate(&mut self, components: &mut HashMap<Id, Box<dyn Component>>);
    fn focus_next(&mut self, current_focus: &Id) -> Option<Id>;
    fn focus_previous(&mut self, current_focus: &Id) -> Option<Id>;
    fn draw(
        &mut self,
        frame: &mut Frame,
        components: &mut HashMap<Id, Box<dyn Component>>,
        state: &SuiteDataCollectionRaw,
    ) -> Result<()>;
}
