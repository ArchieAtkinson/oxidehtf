pub(crate) mod actions;
pub(crate) mod app;
pub(crate) mod components;
pub(crate) mod events;
pub(crate) mod test_runner;
pub(crate) mod ui;

pub use test_runner::user_text_input::TextInput;
pub use test_runner::TestContext;

use test_runner::{FuncType, TestData, TestFunctions, TestMetadata, TestState};

use cli_log::*;
use color_eyre::eyre::Result;
use tokio::runtime::Runtime;

pub fn gen_test_data(funcs: Vec<FuncType>, names: Vec<&'static str>) -> (TestFunctions, TestData) {
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

pub fn run_tests(funcs: TestFunctions, data: TestData) -> Result<()> {
    init_cli_log!();

    let rt = Runtime::new()?;

    info!("Starting");

    rt.block_on(async move {
        let mut ui = app::App::new(funcs, data)?;
        ui.run().await
    })?;

    info!("Finish");

    Ok(())
}
