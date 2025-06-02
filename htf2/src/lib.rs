pub(crate) mod actions;
pub(crate) mod app;
pub(crate) mod components;
pub(crate) mod context;
pub(crate) mod errors;
pub(crate) mod events;
pub(crate) mod lifecycle;
pub(crate) mod measurement;
pub(crate) mod test_runner;
pub(crate) mod ui;

pub use context::SysContext;
pub use errors::TestFailure;
pub use lifecycle::TestLifecycle;
pub use measurement::Unit;

use cli_log::*;
use color_eyre::eyre::Result;
use context::user_text_input::TextInput;
use measurement::Measurements;
use std::sync::Arc;
use test_runner::{FuncType, TestData, TestFunctions, TestMetadata, TestRunner, TestState};
use tokio::{
    runtime::Runtime,
    sync::{mpsc, RwLock},
};

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
                user_inputs: Vec::new(),
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
            text_input: TextInput::new(event_tx.clone(), input_rx),
            measurements: Measurements::new(),
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
