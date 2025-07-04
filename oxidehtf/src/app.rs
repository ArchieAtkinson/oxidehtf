use std::collections::VecDeque;

use crate::{
    common::*,
    test_runner::{
        data::suite::SuiteDataCollection, SuiteData, SuiteProducer, SuiteProducerGenerator,
        TestDone, TestRunner, TestState,
    },
    ui::{Screens, Ui},
};
use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    #[default]
    WaitingForInput,
    Done,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    WelcomeIntro,
    WelcomeSuites,
    RunningSuiteProgress,
    RunningTextInput,
    RunningCurrentTest,
    RunningCompletedTests,
    RunningWaitingTests,
}

pub struct App {
    ui: Ui,
    state: AppState,
    suites_data: SuiteDataCollection,
    test_runner: Option<TestRunner>,
    actions: VecDeque<Action>,
    to_test_runner_tx: UnboundedSender<Action>,
    event_rx: UnboundedReceiver<Event>,
}

impl App {
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = unbounded_channel();
        let (to_test_runner_tx, to_test_runner_rx) = unbounded_channel();

        let mut builders = inventory::iter::<SuiteProducerGenerator>
            .into_iter()
            .collect::<Vec<&SuiteProducerGenerator>>();

        builders.sort_by(|a, b| a.prio.cmp(&b.prio));

        let (data, executors): (Vec<SuiteData>, Vec<Box<dyn SuiteProducer>>) = builders
            .iter()
            .map(|p| {
                let executor = (p.func)();
                let names = executor.get_tests().iter().map(|t| t.0).collect();
                (
                    SuiteData::new(names, executor.get_suite_name(), p.prio),
                    executor,
                )
            })
            .collect();

        let suites_collection = SuiteDataCollection::new(data, event_tx.clone());

        let test_runner = TestRunner::new(
            executors,
            suites_collection.clone(),
            event_tx.clone(),
            to_test_runner_rx,
        );

        Ok(Self {
            ui: Ui::new(event_tx.clone()),
            suites_data: suites_collection,
            state: Default::default(),
            test_runner: Some(test_runner),
            actions: VecDeque::new(),
            to_test_runner_tx,
            event_rx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.ui.start();
        let mut test_runner = self.test_runner.take().ok_or_eyre("No Test Runner")?;
        let mut runner_handle = tokio::task::spawn_blocking(move || test_runner.run());
        let mut is_runner_done = false;

        while self.state() != AppState::Done {
            self.handle_event().await?;
            self.handle_actions().await?;
            let state = self.suites_data.get_raw_copy().await;
            self.ui.render(state)?;

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

        self.produce_junit_report().await?;

        Ok(())
    }

    async fn handle_event(&mut self) -> Result<()> {
        let Some(mut event) = self.event_rx.recv().await else {
            return Ok(());
        };

        let action = match event {
            Event::Key(key) => match (key.modifiers, key.code) {
                (_, KeyCode::Esc) => Some(Action::ExitApp),
                (KeyModifiers::NONE, KeyCode::Tab) => Some(Action::FocusNextPane),
                (KeyModifiers::SHIFT, KeyCode::BackTab) => Some(Action::FocusPreviousPane),
                _ => None,
            },
            Event::UserInputPrompt(ref s, ref mut c) => {
                let channel = c.take();
                Some(Action::UserInputPrompt(s.clone(), channel))
            }
            Event::CurrentSuiteDut(ref s) => Some(Action::SetCurrentSuiteDut(s.clone())),
            Event::TestsCompleted => Some(Action::ChangeScreen(Screens::Summary)),
            // Used to update UI
            Event::NOP => return Ok(()),
            _ => None,
        };

        if let Some(action) = action {
            self.actions.push_back(action);
        }

        if let Some(component) = self.ui.focused_component() {
            if let Some(action) = component.handle_event(&event)? {
                self.actions.push_back(action);
            }
        }
        Ok(())
    }

    async fn handle_actions(&mut self) -> Result<()> {
        use Action::*;

        while let Some(mut action) = self.actions.pop_front() {
            match action {
                ExitApp => self.state = AppState::Done,
                FocusNextPane => self.ui.focus_next(),
                FocusPreviousPane => self.ui.focus_previous(),
                ChangeScreen(s) => {
                    if s == Screens::RunningTests {
                        self.actions.push_back(Action::StartTests);
                    }
                    self.ui.active(s);
                }
                SetCurrentSuiteDut(ref s) => {
                    self.suites_data
                        .write(|d| {
                            d.dut_id = s.clone();
                            Ok(())
                        })
                        .await?
                }
                _ => (),
            }

            if let Some(component) = self.ui.focused_component() {
                if let Some(action) = component.update(&mut action)? {
                    self.actions.push_back(action);
                }
            }

            let _ = self.to_test_runner_tx.send(action);
        }

        Ok(())
    }

    fn state(&self) -> AppState {
        self.state
    }

    // fn focus_default(&mut self) -> Result<()> {
    //     self.active_components()?[0].focus();
    //     self.current_focus = 0;
    //     Ok(())
    // }

    // fn focus_next(&mut self) -> Result<()> {
    //     let current_focus = self.current_focus;
    //     self.active_components()?[current_focus].blur();

    //     let len = self.active_components()?.len();

    //     let start_search_index = self.current_focus + 1;
    //     let mut next_focus_index = 0;

    //     let mut found_next_focusable = false;

    //     for i in 0..len {
    //         let index = (start_search_index + i) % len;
    //         if self.active_components()?[index].can_focus() {
    //             next_focus_index = index;
    //             found_next_focusable = true;
    //             break;
    //         }
    //     }

    //     if found_next_focusable {
    //         self.active_components()?[next_focus_index].focus();
    //         self.current_focus = next_focus_index;
    //     } else {
    //         panic!(
    //             "No other focusable components found in the sequence. Current focus remains {}.",
    //             self.current_focus
    //         );
    //     }
    //     Ok(())
    // }

    // fn focus_previous(&mut self) -> Result<()> {
    //     let current_focus = self.current_focus;
    //     self.active_components()?[current_focus].blur();

    //     let len = self.active_components()?.len();

    //     let start_search_index = self.current_focus;
    //     let mut next_focus_index = 0;

    //     let mut found_next_focusable = false;

    //     for i in 0..len {
    //         let index = (start_search_index + len - 1 - i) % len;
    //         if self.active_components()?[index].can_focus() {
    //             next_focus_index = index;
    //             found_next_focusable = true;
    //             break;
    //         }
    //     }

    //     if found_next_focusable {
    //         self.active_components()?[next_focus_index].focus();
    //         self.current_focus = next_focus_index;
    //     } else {
    //         panic!(
    //             "No other focusable components found in the sequence. Current focus remains {}.",
    //             self.current_focus
    //         );
    //     }

    //     Ok(())
    // }

    async fn produce_junit_report(&self) -> Result<()> {
        use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite};

        let mut report = Report::new("htf2-run");

        for suite in &self.suites_data.data.read().await.inner {
            let mut test_suite = TestSuite::new(format!("{}", suite.name));

            for test in &suite.test_data {
                let test_case_result = match &test.state {
                    TestState::Done(r) => match r {
                        TestDone::Passed => TestCaseStatus::success(),
                        TestDone::Failed(_) => TestCaseStatus::non_success(NonSuccessKind::Failure),
                    },

                    _ => TestCaseStatus::non_success(NonSuccessKind::Error),
                };
                let mut test_case = TestCase::new(test.name, test_case_result);
                test_case.set_time(test.duration);
                test_suite.add_test_case(test_case);
            }

            report.add_test_suite(test_suite);
            report.timestamp = Some(suite.start_time);
        }

        let junit_file = std::fs::File::create("junit-report.xml")?;

        report.serialize(junit_file)?;

        Ok(())
    }
}
