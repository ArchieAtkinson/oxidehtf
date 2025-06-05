pub(crate) mod actions;
pub(crate) mod app;
pub(crate) mod components;
pub(crate) mod events;
pub(crate) mod test_runner;
pub(crate) mod ui;

use cli_log::*;
use color_eyre::eyre::Result;
use test_runner::test_data::TestDataManager;
use test_runner::{
    context::{dut::DUT, measurement::Measurements, user_text_input::TextInput},
    FuncType, TestFunctions, TestRunner,
};
use tokio::{runtime::Runtime, sync::mpsc};

pub use test_runner::context::{measurement::Unit, SysContext};
pub use test_runner::errors::TestFailure;
pub use test_runner::lifecycle::TestLifecycle;

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

#[macro_export]
macro_rules! register_tests {
    ($($func_name:ident),*) => {
        (vec![$($func_name),*], vec![$(stringify!($func_name)),*]);
    };
}

pub fn run_tests<T: Send + 'static + TestLifecycle>(
    funcs: Vec<FuncType<T>>,
    names: Vec<&'static str>,
    fixture: T,
) -> Result<()> {
    init_cli_log!();

    let rt = Runtime::new()?;

    info!("Starting");

    rt.block_on(async move {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (input_tx, input_rx) = mpsc::unbounded_channel();

        let test_funcs = TestFunctions { funcs };
        let test_data = TestDataManager::new(names, event_tx.clone());

        let context = SysContext {
            text_input: TextInput::new(event_tx.clone(), input_rx, test_data.clone()),
            measurements: Measurements::new(test_data.clone()),
            dut: DUT::new(test_data.clone()),
        };

        let mut test_runner = TestRunner::new(
            test_funcs,
            test_data.clone(),
            event_tx.clone(),
            context,
            fixture,
        );

        tokio::task::spawn_blocking(move || test_runner.run());

        let mut app = app::App::new(test_data.clone(), event_rx, event_tx, input_tx)?;

        app.run().await
    })?;

    info!("Finish");

    Ok(())
}
