pub(crate) mod actions;
pub(crate) mod app;
pub(crate) mod common;
pub(crate) mod components;
pub(crate) mod event_handlers;
pub(crate) mod events;
pub(crate) mod test_runner;
pub(crate) mod ui;

use common::*;

pub use test_runner::context::measurement::Unit;
pub use test_runner::SysContext;
pub use test_runner::TestFailure;
pub use test_runner::TestLifecycle;
pub use test_runner::TestSuiteInventory;
pub use test_runner::TestSuiteInventoryFactory;

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr, $($arg:tt)*) => {{
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    return Err(oxidehtf::TestFailure::AssertionFailed {
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
        oxidehtf::assert_eq!($left, $right, "");
    }};
}

#[macro_export]
macro_rules! register_tests {
    ($($func_name:ident),*) => {
        (vec![$($func_name),*], vec![$(stringify!($func_name)),*]);
    };
}

pub fn run_tests() -> Result<()> {
    init_cli_log!();

    let rt = tokio::runtime::Runtime::new()?;

    info!("Starting");

    let factory = inventory::iter::<TestSuiteInventoryFactory>
        .into_iter()
        .nth(0)
        .unwrap();

    let suite = (factory.func)();

    rt.block_on(async move {
        let mut app = app::App::new(suite)?;
        app.run().await
    })?;

    info!("Finish");

    Ok(())
}
