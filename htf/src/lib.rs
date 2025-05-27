pub(crate) mod actions;
pub(crate) mod components;
pub(crate) mod events;
pub(crate) mod ui;

pub use components::operator::Input;
pub use components::test_runner::Test;

use cli_log::*;
use color_eyre::eyre::Result;
use tokio::runtime::Runtime;
use ui::Ui;

#[macro_export]
macro_rules! register_tests {
    ($($func_name:ident),*) => {
        vec![
            $(
            htf::Test::new($func_name, stringify!($func_name))
            ),*
        ]
    };
}

pub fn run_tests(tests: Vec<Test>) -> Result<()> {
    init_cli_log!();

    let rt = Runtime::new()?;

    info!("Starting");

    rt.block_on(async move {
        let mut ui = Ui::new(tests)?;
        ui.run().await
    })?;

    info!("Finish");

    Ok(())
}
