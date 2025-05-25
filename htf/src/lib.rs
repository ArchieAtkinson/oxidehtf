pub mod events;
pub mod test_runner;
pub mod ui;

use crate::test_runner::OperatorPrompt;
use crate::test_runner::{TestMetadata, TestRunner};
use crate::ui::{AppState, Model};

use cli_log::*;
use color_eyre::eyre::Result;
use test_runner::{OperatorInput, Test};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub fn run_tests(tests: Vec<Test>) -> Result<()> {
    init_cli_log!();

    let (test_data_send, test_data_recv) = mpsc::unbounded_channel::<TestMetadata>();
    let (req_input_send, req_input_recv) = mpsc::unbounded_channel::<OperatorPrompt>();
    let (op_input_send, op_input_recv) = mpsc::unbounded_channel::<OperatorInput>();

    let mut runner = TestRunner::new(test_data_send, req_input_send, op_input_recv, tests)?;
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let test_handle = tokio::task::spawn_blocking(move || {
            runner.run();
            runner
        });

        let app_handle =
            tokio::spawn(async move { app(test_data_recv, req_input_recv, op_input_send).await });

        let (_runner, _app) = tokio::join!(test_handle, app_handle);
    });

    Ok(())
}

async fn app(
    test_recivier: mpsc::UnboundedReceiver<TestMetadata>,
    prompt_recivier: mpsc::UnboundedReceiver<OperatorPrompt>,
    input_sender: mpsc::UnboundedSender<OperatorInput>,
) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut model = Model::new(test_recivier, prompt_recivier, input_sender);

    while model.mode() != AppState::Done {
        if let Some(msg) = model.handle_event().await? {
            model.update(msg).await?;
        }

        terminal.draw(|f| model.view(f))?;
    }
    ratatui::restore();
    Ok(())
}
