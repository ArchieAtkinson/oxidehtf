use crate::{common::*, test_runner::SuiteDataCollectionRaw};
use ratatui::{
    layout::{Constraint, Rect},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Attribute {
    VertConstraint(Option<Constraint>),
    Focus(Option<bool>),
}

pub trait Component {
    // fn init(&mut self) -> Result<()> {
    //     Ok(())
    // }

    #[allow(dead_code)]
    fn name(&self) -> &str;

    fn register_event_handler(&mut self, tx: UnboundedSender<Event>) -> Result<()> {
        let _ = tx;
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<Option<Action>> {
        let _ = event;
        Ok(None)
    }

    fn update(&mut self, action: &mut Action) -> Result<Option<Action>> {
        let _ = action;
        Ok(None)
    }

    fn get_attr(&self, _attr: Attribute) -> Result<Attribute> {
        Err(eyre!("Get Attr not impl on component {}", self.name()))
    }

    fn set_attr(&mut self, _attr: Attribute) -> Result<()> {
        Err(eyre!("Set Attr not impl on component {}", self.name()))
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, state: &SuiteDataCollectionRaw)
        -> Result<()>;
}
