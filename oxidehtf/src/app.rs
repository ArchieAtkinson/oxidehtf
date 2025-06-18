use std::collections::HashMap;

use crate::{common::*, TestSuiteInventory};
use crossterm::event::{KeyCode, KeyModifiers};

use crate::{
    components::{
        CompletedTestDisplay, Component, CurrentTestDisplay, SuiteProgressDisplay, UserTextInput,
        WaitingTestDisplay, WeclomeDisplay,
    },
    test_runner::{FuncType, SuiteData, TestDone, TestRunner, TestState},
    ui::Ui,
    SysContext, TestLifecycle,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    #[default]
    WaitingForInput,
    Done,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    Welcome,
    RunningTests,
}

pub struct App {
    ui: Ui,
    state: AppState,
    suite_data: SuiteData,
    components: HashMap<Screen, Vec<Box<dyn Component>>>,
    test_runner: Option<TestRunner>,
    current_focus: usize,
    current_screen: Screen,
    action_rx: UnboundedReceiver<Action>,
    action_tx: UnboundedSender<Action>,
    event_rx: UnboundedReceiver<Event>,
    event_tx: UnboundedSender<Event>,
    input_tx: UnboundedSender<String>,
}

impl App {
    pub fn new(inventory: TestSuiteInventory) -> Result<Self> {
        let (action_tx, action_rx) = unbounded_channel();
        let (event_tx, event_rx) = unbounded_channel();
        let (input_tx, input_rx) = unbounded_channel();

        let suite_data = SuiteData::new(inventory.names.clone(), event_tx.clone());

        let context = SysContext::new(suite_data.clone(), event_tx.clone(), input_rx);
        let test_runner = TestRunner::new(
            inventory.executer,
            suite_data.clone(),
            event_tx.clone(),
            context,
        );

        let running_tests_screen: Vec<Box<dyn Component>> = vec![
            // User text input first to start as focus
            Box::new(UserTextInput::new()),
            Box::new(SuiteProgressDisplay::new()),
            Box::new(WaitingTestDisplay::new()),
            Box::new(CompletedTestDisplay::new()),
            Box::new(CurrentTestDisplay::new()),
        ];

        let welcome_screen: Vec<Box<dyn Component>> = vec![Box::new(WeclomeDisplay::new())];

        Ok(Self {
            ui: Ui::new(event_tx.clone()),
            suite_data,
            components: HashMap::from([
                (Screen::RunningTests, running_tests_screen),
                (Screen::Welcome, welcome_screen),
            ]),
            test_runner: Some(test_runner),
            current_focus: 0,
            current_screen: Screen::Welcome,
            state: Default::default(),
            action_rx,
            action_tx,
            event_rx,
            event_tx,
            input_tx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut test_runner = self.test_runner.take().ok_or_eyre("No Test Runner")?;
        let mut runner_handle = tokio::task::spawn_blocking(move || test_runner.run());
        let mut is_runner_done = false;

        for component in self.components.values_mut().flat_map(|v| v.iter_mut()) {
            component.register_event_handler(self.event_tx.clone())?;
            component.register_action_handler(self.action_tx.clone())?;
            component.init()?;
        }

        self.ui.start();

        let current_focus = self.current_focus;
        self.active_components()?[current_focus].focus();

        info!("Spawning Test Runner");

        while self.state() != AppState::Done {
            self.handle_event().await?;
            self.handle_actions().await?;
            let state = self.suite_data.get_raw_copy().await;
            self.ui.render(|f, a| {
                for component in self
                    .components
                    .get_mut(&self.current_screen)
                    .ok_or_eyre("Screen not present")?
                    .iter_mut()
                {
                    component.draw(f, &a, &state)?;
                }
                Ok(())
            })?;

            if !is_runner_done {
                tokio::select! {
                    result = (&mut runner_handle) => {
                        is_runner_done = true;
                        match result {
                            Ok(_) =>  info!("Runner handle completed successfully!"),
                            Err(e) => info!("Runner handle failed: {:?}", e),
                        }
                    },
                    _ = tokio::time::sleep(tokio::time::Duration::from_nanos(1)) => {
                    }
                }
            }
        }

        Self::produce_junit_report(&self.suite_data).await?;

        Ok(())
    }

    async fn handle_event(&mut self) -> Result<()> {
        let Some(event) = self.event_rx.recv().await else {
            return Ok(());
        };

        let action = match event.clone() {
            Event::Key(key) => match (key.modifiers, key.code) {
                (_, KeyCode::Esc) => Some(Action::ExitApp),
                (KeyModifiers::NONE, KeyCode::Tab) => Some(Action::FocusNextPane),
                (KeyModifiers::SHIFT, KeyCode::BackTab) => Some(Action::FocusPreviousPane),
                _ => None,
            },
            Event::UserInputPrompt(s) => Some(Action::UserInputPrompt(s)),
            _ => None,
        };

        if let Some(action) = action {
            self.action_tx.send(action)?;
        }

        for component in self
            .components
            .get_mut(&self.current_screen)
            .ok_or_eyre("Screen not present")?
            .iter_mut()
        {
            if let Some(new_action) = component.handle_events(event.clone())? {
                self.action_tx.send(new_action)?;
            }
        }

        Ok(())
    }

    async fn handle_actions(&mut self) -> Result<()> {
        use Action::*;

        while let Ok(action) = self.action_rx.try_recv() {
            match action.clone() {
                ExitApp => self.state = AppState::Done,
                FocusNextPane => self.focus_next()?,
                FocusPreviousPane => self.focus_previous()?,
                UserInputValue(s) => self.input_tx.send(s)?,
                ChangeScreen(s) => {
                    self.current_screen = s;
                    self.focus_default()?;
                }
                _ => (),
            }

            for component in self
                .components
                .get_mut(&self.current_screen)
                .ok_or_eyre("Screen not present")?
                .iter_mut()
            {
                if let Some(new_action) = component.update(action.clone())? {
                    self.action_tx.send(new_action)?;
                }
            }
        }

        Ok(())
    }

    fn state(&self) -> AppState {
        self.state
    }

    fn focus_default(&mut self) -> Result<()> {
        self.active_components()?[0].focus();
        self.current_focus = 0;
        Ok(())
    }

    fn focus_next(&mut self) -> Result<()> {
        let current_focus = self.current_focus;
        self.active_components()?[current_focus].blur();

        let len = self.active_components()?.len();

        let start_search_index = self.current_focus + 1;
        let mut next_focus_index = 0;

        let mut found_next_focusable = false;

        for i in 0..len {
            let index = (start_search_index + i) % len;
            if self.active_components()?[index].can_focus() {
                next_focus_index = index;
                found_next_focusable = true;
                break;
            }
        }

        if found_next_focusable {
            self.active_components()?[next_focus_index].focus();
            self.current_focus = next_focus_index;
        } else {
            panic!(
                "No other focusable components found in the sequence. Current focus remains {}.",
                self.current_focus
            );
        }
        Ok(())
    }

    fn focus_previous(&mut self) -> Result<()> {
        let current_focus = self.current_focus;
        self.active_components()?[current_focus].blur();

        let len = self.active_components()?.len();

        let start_search_index = self.current_focus;
        let mut next_focus_index = 0;

        let mut found_next_focusable = false;

        for i in 0..len {
            let index = (start_search_index + len - 1 - i) % len;
            if self.active_components()?[index].can_focus() {
                next_focus_index = index;
                found_next_focusable = true;
                break;
            }
        }

        if found_next_focusable {
            self.active_components()?[next_focus_index].focus();
            self.current_focus = next_focus_index;
        } else {
            panic!(
                "No other focusable components found in the sequence. Current focus remains {}.",
                self.current_focus
            );
        }

        Ok(())
    }

    async fn produce_junit_report(data: &SuiteData) -> Result<()> {
        use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite};

        let mut report = Report::new("htf2-run");
        let mut test_suite = TestSuite::new("htf2-suite");

        let data = data.get_raw_copy().await;

        for test in data.test_metadata {
            let test_case_result = match test.state {
                TestState::Done(r) => match r {
                    TestDone::Passed => TestCaseStatus::success(),
                    TestDone::Failed => TestCaseStatus::non_success(NonSuccessKind::Failure),
                },

                _ => TestCaseStatus::non_success(NonSuccessKind::Error),
            };
            let mut test_case = TestCase::new(test.name, test_case_result);
            test_case.set_time(test.duration);
            test_suite.add_test_case(test_case);
        }

        report.add_test_suite(test_suite);
        report.timestamp = Some(data.start_time);

        let junit_file = std::fs::File::create("junit-report.xml")?;

        report.serialize(junit_file)?;

        Ok(())
    }

    fn active_components(&mut self) -> Result<&mut Vec<Box<dyn Component>>> {
        self.components
            .get_mut(&self.current_screen)
            .ok_or_eyre("No Screen Present")
    }
}
