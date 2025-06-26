// let running_tests_screen: Vec<Box<dyn Component>> = vec![
//     // User text input first to start as focus
//     Box::new(UserTextInput::new()),
//     Box::new(SuiteProgressDisplay::new()),
//     Box::new(WaitingTestDisplay::new()),
//     Box::new(CompletedTestDisplay::new()),
//     Box::new(CurrentTestDisplay::new()),
// ];

use super::components::Component;
use super::Screen;
use crate::{app::Id, common::*, test_runner::SuiteDataCollectionRaw};
use completed_tests::CompletedTestDisplay;
use current_test::CurrentTestDisplay;
use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};
use suite_progress::SuiteProgressDisplay;
use user_text_input::UserTextInput;
use waiting_tests::WaitingTestDisplay;

pub mod completed_tests;
pub mod current_test;
pub mod suite_progress;
pub mod user_text_input;
pub mod waiting_tests;

pub struct RunningScreen {
    event_tx: UnboundedSender<Event>,
}

impl RunningScreen {
    pub fn new(event_tx: UnboundedSender<Event>) -> Self {
        Self { event_tx }
    }
}

impl Screen for RunningScreen {
    fn name(&self) -> &str {
        "Running"
    }

    fn activate(
        &mut self,
        components: &mut std::collections::HashMap<crate::app::Id, Box<dyn Component>>,
    ) -> Option<Id> {
        components.insert(
            Id::RunningSuiteProgress,
            Box::new(SuiteProgressDisplay::new()),
        );
        components.insert(Id::RunningTextInput, Box::new(UserTextInput::new()));
        components.insert(Id::RunningCurrentTest, Box::new(CurrentTestDisplay::new()));
        components.insert(Id::RunningWaitingTests, Box::new(WaitingTestDisplay::new()));
        components.insert(
            Id::RunningCompletedTests,
            Box::new(CompletedTestDisplay::new()),
        );

        Some(Id::RunningTextInput)
    }

    fn deactivate(&mut self, components: &mut std::collections::HashMap<Id, Box<dyn Component>>) {
        components.remove(&Id::RunningSuiteProgress);
        components.remove(&Id::RunningTextInput);
        components.remove(&Id::RunningCurrentTest);
        components.remove(&Id::RunningWaitingTests);
        components.remove(&Id::RunningCompletedTests);
    }

    fn focus_next(&mut self, current_focus: Id) -> Id {
        match current_focus {
            Id::RunningTextInput => Id::RunningCurrentTest,
            Id::RunningCurrentTest => Id::RunningTextInput,
            _ => panic!("Can't focus next from unknown ID"),
        }
    }

    fn focus_previous(&mut self, current_focus: Id) -> Id {
        match current_focus {
            Id::RunningTextInput => Id::RunningCurrentTest,
            Id::RunningCurrentTest => Id::RunningTextInput,
            _ => panic!("Can't focus next from unknown ID"),
        }
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        components: &mut std::collections::HashMap<Id, Box<dyn Component>>,
        state: &SuiteDataCollectionRaw,
    ) -> Result<()> {
        let [test_progress, operator, current_test, lists_of_tests] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(1),
        ])
        .areas(frame.area());

        let [completed_list, waiting_list] =
            Layout::vertical([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                .areas(lists_of_tests);

        components
            .get_mut(&Id::RunningSuiteProgress)
            .unwrap()
            .draw(frame, test_progress, state)?;

        components
            .get_mut(&Id::RunningTextInput)
            .unwrap()
            .draw(frame, operator, state)?;

        components
            .get_mut(&Id::RunningCurrentTest)
            .unwrap()
            .draw(frame, current_test, state)?;

        components
            .get_mut(&Id::RunningWaitingTests)
            .unwrap()
            .draw(frame, waiting_list, state)?;

        components
            .get_mut(&Id::RunningCompletedTests)
            .unwrap()
            .draw(frame, completed_list, state)?;

        Ok(())
    }
}
