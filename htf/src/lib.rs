pub(crate) mod actions;
pub(crate) mod component;
pub(crate) mod events;
pub mod operator;
pub mod test_runner;
pub(crate) mod ui;

use crate::test_runner::{TestMetadata, TestRunner};
use crate::ui::{AppState, Ui};

use cli_log::*;
use color_eyre::eyre::{eyre, Result};
use test_runner::Test;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub fn run_tests(tests: Vec<Test>) -> Result<()> {
    init_cli_log!();

    let (test_data_send, test_data_recv) = mpsc::unbounded_channel::<TestMetadata>();

    let mut runner = TestRunner::new(test_data_send, tests)?;
    let mut ui = Ui::new(test_data_recv)?;

    let rt = Runtime::new()?;

    rt.block_on(async {
        let test_handle: JoinHandle<Result<()>> = tokio::task::spawn_blocking(move || {
            runner.run()?;
            Ok(())
        });

        let app_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
            let mut terminal = ratatui::init();

            while ui.mode() != AppState::Done {
                if let Some(msg) = ui.handle_event().await? {
                    ui.update(msg).await?;
                }
                let mut result = Ok(());
                terminal.draw(|f| result = ui.view(f))?;
                if result.is_err() {
                    return result;
                }
            }
            ratatui::restore();
            Ok(())
        });

        let (_runner, _app) = tokio::join!(test_handle, app_handle);
    });

    Ok(())
}
