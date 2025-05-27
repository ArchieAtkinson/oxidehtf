// use cli_log::*;
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, LineGauge, List},
    Frame,
};
use tokio::sync::mpsc;

use crate::{actions::Action, component::Component, events::Event, ui::UiArea};

#[macro_export]
macro_rules! register_test {
    ($($func_name:ident),*) => {
        vec![
            $(
                htf::test_runner::Test {
                    func: $func_name,
                    data: htf::test_runner::TestMetadata {
                        name: stringify!($func_name),
                        state: htf::test_runner::TestState::Waiting,
                    },
                }
            ),*
        ]
    };
}

pub struct TestTask {
    tests: Vec<Test>,
    tx: mpsc::UnboundedSender<Event>,
}

impl TestTask {
    pub fn run(mut self) -> Result<()> {
        for test in &mut self.tests {
            test.data.state = TestState::Running;
            self.tx.send(Event::TestData(test.data.clone()))?;
            let result = (test.func)();
            test.data.state = match result {
                Ok(_) => TestState::Passed,
                Err(_) => TestState::Failed,
            };
            self.tx.send(Event::TestData(test.data.clone()))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Test {
    pub func: fn() -> Result<()>,
    pub data: TestMetadata,
}

#[derive(Debug, Clone)]
pub struct TestMetadata {
    pub name: &'static str,
    pub state: TestState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestState {
    Waiting,
    Running,
    Passed,
    Failed,
}

pub struct TestRunner {
    tests: Vec<Test>,
    action_tx: Option<mpsc::UnboundedSender<Action>>,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl TestRunner {
    pub fn new(tests: Vec<Test>, event_tx: mpsc::UnboundedSender<Event>) -> Result<Self> {
        Ok(Self {
            tests,
            action_tx: None,
            event_tx,
        })
    }

    pub fn test_task(&self) -> TestTask {
        TestTask {
            tests: self.tests.clone(),
            tx: self.event_tx.clone(),
        }
    }

    fn update_tests(&mut self, data: TestMetadata) -> Result<()> {
        for item in self.tests.iter_mut() {
            if item.data.name == data.name {
                item.data.state = data.state.clone();
                break;
            }
        }

        Ok(())
    }

    fn render_progress(&self, frame: &mut Frame, area: Rect) {
        let tests_finished = self
            .tests
            .iter()
            .filter(|test| {
                test.data.state == TestState::Passed || test.data.state == TestState::Failed
            })
            .count() as f64;

        let total_tests = self.tests.len() as f64;

        let mut progress: f64 = tests_finished / total_tests;
        if total_tests == 0.0 {
            progress = 0.0;
        }

        let bar = LineGauge::default()
            .filled_style(Style::new().white().on_black().bold())
            .block(Block::bordered().title("Progress"))
            .ratio(progress);
        frame.render_widget(bar, area);
    }

    fn render_messages(&self, frame: &mut Frame, area: Rect) {
        let messages = self
            .tests
            .iter()
            .enumerate()
            .map(|(i, test)| format!("{}: {} {:?}", i, test.data.name, test.data.state));
        let messages = List::new(messages).block(Block::bordered().title("Messages"));
        frame.render_widget(messages, area);
    }
}

impl Component for TestRunner {
    fn init(&mut self) {
        let task_runner = self.test_task();
        tokio::task::spawn_blocking(move || task_runner.run());
    }

    fn draw(&mut self, frame: &mut Frame, area: &UiArea) -> Result<()> {
        assert_eq!(area.test_progress.height, 3);

        self.render_progress(frame, area.test_progress);
        self.render_messages(frame, area.test_list);

        Ok(())
    }

    fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx.clone());

        Ok(())
    }

    fn handle_events(&mut self, event: crate::events::Event) -> Result<Option<Action>> {
        match event {
            Event::TestData(d) => Ok(Some(Action::TestUpdate(d))),
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::TestUpdate(d) => {
                self.update_tests(d)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
