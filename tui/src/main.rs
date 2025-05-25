mod app;
mod events;
mod test_runner;

use app::{AppState, Model};
use cli_log::*;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use test_runner::OperatorComms;
use test_runner::{TestMetadata, TestRunner};
use tokio::sync::mpsc;

fn test1(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 1 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

fn test2(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 2 Input:".to_string());
    info!("{:?}", value);
    Err(eyre!("Err"))
}

fn test3(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 3 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

fn test4(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 4 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

fn test5(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 5 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

fn test6(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 6 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

#[derive(Debug)]
struct OperatorPrompt(String);

#[derive(Debug)]
struct OperatorInput(String);

#[tokio::main]
async fn main() -> Result<()> {
    init_cli_log!();

    let (test_data_send, test_data_recv) = mpsc::unbounded_channel::<TestMetadata>();
    let (req_input_send, req_input_recv) = mpsc::unbounded_channel::<OperatorPrompt>();
    let (op_input_send, op_input_recv) = mpsc::unbounded_channel::<OperatorInput>();

    let tests = register_test!(test1, test2, test3, test4, test5, test6);
    let mut runner = TestRunner::new(test_data_send, req_input_send, op_input_recv, tests)?;

    let test_handle = tokio::task::spawn_blocking(move || {
        runner.run();
        runner
    });

    let app_handle =
        tokio::spawn(async move { app(test_data_recv, req_input_recv, op_input_send).await });

    let (_runner, _app) = tokio::join!(test_handle, app_handle);
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
