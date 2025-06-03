pub(crate) mod actions;
pub(crate) mod app;
pub(crate) mod components;
pub(crate) mod events;
pub(crate) mod test_runner;
pub(crate) mod ui;

use cli_log::*;
use color_eyre::eyre::Result;
use indexmap::IndexMap;
use std::sync::Arc;
use test_runner::{
    context::{measurement::Measurements, user_text_input::TextInput},
    FuncType, TestData, TestFunctions, TestMetadata, TestRunner, TestState,
};
use tokio::{
    runtime::Runtime,
    sync::{mpsc, RwLock},
};

pub use test_runner::context::{measurement::Unit, SysContext};
pub use test_runner::errors::TestFailure;
pub use test_runner::lifecycle::TestLifecycle;

pub fn gen_test_data<T>(
    funcs: Vec<FuncType<T>>,
    names: Vec<&'static str>,
) -> (TestFunctions<T>, TestData) {
    let test_funcs = TestFunctions { funcs };

    let test_data = TestData {
        data: names
            .iter()
            .map(|n| TestMetadata {
                name: *n,
                state: TestState::InQueue,
                user_inputs: IndexMap::new(),
                measurements: IndexMap::new(),
            })
            .collect(),
        current_index: 0,
    };

    (test_funcs, test_data)
}

#[macro_export]
macro_rules! register_tests {
    ($($func_name:ident),*) => {
        htf2::gen_test_data(
            vec![$($func_name),*],
            vec![$(stringify!($func_name)),*]
        )
    };
}

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr, $($arg:tt)*) => {{
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    return Err(htf2::TestFailure::AssertionFailed {
                        expected: format!("{:?}", right_val),
                        found: format!("{:?}", left_val),
                        file: file!(),
                        line: line!(),
                    });
                }
            }
        }
    }};
    ($left:expr, $right:expr) => {{
        htf2::assert_eq!($left, $right, "");
    }};
}

pub fn run_tests<T: Send + 'static + TestLifecycle>(
    funcs: TestFunctions<T>,
    data: TestData,
    fixture: T,
) -> Result<()> {
    init_cli_log!();

    let rt = Runtime::new()?;

    info!("Starting");

    rt.block_on(async move {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (input_tx, input_rx) = mpsc::unbounded_channel();

        let test_data = Arc::new(RwLock::new(data));

        let context = SysContext {
            text_input: TextInput::new(event_tx.clone(), input_rx, test_data.clone()),
            measurements: Measurements::new(test_data.clone(), event_tx.clone()),
        };

        let mut test_runner =
            TestRunner::new(funcs, test_data.clone(), event_tx.clone(), context, fixture);

        tokio::task::spawn_blocking(move || test_runner.run());

        let mut app = app::App::new(test_data.clone(), event_rx, event_tx, input_tx)?;

        app.run().await
    })?;

    info!("Finish");

    Ok(())
}
