pub(crate) mod events;
pub mod operator;
pub mod test_runner;
pub(crate) mod ui;

use crate::test_runner::{TestMetadata, TestRunner};
use crate::ui::{AppState, Model};

use cli_log::*;
use color_eyre::eyre::Result;
use test_runner::Test;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub fn run_tests(tests: Vec<Test>) -> Result<()> {
    init_cli_log!();

    let (test_data_send, test_data_recv) = mpsc::unbounded_channel::<TestMetadata>();
    let ui_op_comms = operator::init()?;
    let mut runner = TestRunner::new(test_data_send, tests)?;

    let rt = Runtime::new()?;

    rt.block_on(async {
        let test_handle: JoinHandle<Result<()>> = tokio::task::spawn_blocking(move || {
            runner.run()?;
            Ok(())
        });

        let app_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
            let mut terminal = ratatui::init();
            let mut model = Model::new(test_data_recv, ui_op_comms);

            while model.mode() != AppState::Done {
                if let Some(msg) = model.handle_event().await? {
                    model.update(msg).await?;
                }

                terminal.draw(|f| model.view(f))?;
            }
            ratatui::restore();
            Ok(())
        });

        let (_runner, _app) = tokio::join!(test_handle, app_handle);
    });

    Ok(())
}
