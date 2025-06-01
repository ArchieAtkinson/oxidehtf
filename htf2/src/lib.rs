pub(crate) mod actions;
pub(crate) mod app;
pub(crate) mod components;
pub(crate) mod errors;
pub(crate) mod events;
pub(crate) mod plugs;
pub(crate) mod test_runner;
pub(crate) mod ui;

pub use errors::TestFailure;
pub use events::Event;
pub use plugs::user_text_input::TextInput;
pub use plugs::Plug;

use std::sync::Arc;

use test_runner::{FuncType, TestData, TestFunctions, TestMetadata, TestRunner, TestState};

use cli_log::*;
use color_eyre::eyre::Result;
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

pub fn run_tests<T: Send + 'static + Plug>(
    funcs: TestFunctions<T>,
    data: TestData,
    mut context: T,
) -> Result<()> {
    init_cli_log!();

    let rt = Runtime::new()?;

    info!("Starting");

    rt.block_on(async move {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let test_data = Arc::new(RwLock::new(data));

        context.register_event_handler(event_tx.clone());

        let mut test_runner = TestRunner::new(funcs, test_data.clone(), event_tx.clone(), context);

        tokio::task::spawn_blocking(move || test_runner.run());

        let mut app = app::App::new(test_data.clone(), event_rx, event_tx)?;
        app.run().await
    })?;

    info!("Finish");

    Ok(())
}
