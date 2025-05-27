pub(crate) mod actions;
pub(crate) mod component;
pub(crate) mod events;
pub mod operator;
pub mod test_runner;
pub(crate) mod ui;

use cli_log::*;
use color_eyre::eyre::Result;
use test_runner::Test;
use tokio::runtime::Runtime;
use ui::Ui;

pub fn run_tests(tests: Vec<Test>) -> Result<()> {
    init_cli_log!();

    let mut ui = Ui::new(tests)?;

    let rt = Runtime::new()?;

    info!("Starting");

    rt.block_on(async { tokio::spawn(async move { ui.run().await }).await? })?;

    info!("Finish");

    Ok(())
}
