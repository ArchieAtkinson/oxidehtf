pub(crate) mod actions;
pub(crate) mod app;
pub(crate) mod components;
pub(crate) mod events;
pub(crate) mod test_runner;
pub(crate) mod ui;

pub use test_runner::user_text_input::TextInput;
pub use test_runner::Test;
pub use test_runner::TestContext;

use cli_log::*;
use color_eyre::eyre::Result;
use tokio::runtime::Runtime;

#[macro_export]
macro_rules! register_tests {
    ($($func_name:ident),*) => {
        vec![
            $(
            htf2::Test::new($func_name, stringify!($func_name))
            ),*
        ]
    };
}

pub fn run_tests(tests: Vec<Test>) -> Result<()> {
    init_cli_log!();

    let rt = Runtime::new()?;

    info!("Starting");

    rt.block_on(async move {
        let mut ui = app::App::new(tests)?;
        ui.run().await
    })?;

    info!("Finish");

    Ok(())
}
