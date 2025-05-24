mod app;
mod events;
mod test_runner;

use app::{AppState, Model};
use cli_log::*;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use test_runner::{TestMetadata, TestRunner};
use tokio::sync::mpsc;

fn test1() -> Result<()> {
    Ok(())
}

fn test2() -> Result<()> {
    Err(eyre!("Err"))
}

fn test3() -> Result<()> {
    Ok(())
}

fn test4() -> Result<()> {
    Ok(())
}

fn test5() -> Result<()> {
    Ok(())
}

fn test6() -> Result<()> {
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_cli_log!();

    let (test_data_send, test_data_recv) = mpsc::unbounded_channel::<TestMetadata>();

    let tests = register_test!(test1, test2, test3, test4, test5, test6);
    let mut runner = TestRunner::new(test_data_send, tests)?;

    let handle = tokio::spawn(async move { app(test_data_recv).await });

    let (_runner, _app) = tokio::join!(runner.run(), handle);
    Ok(())
}

async fn app(recv: mpsc::UnboundedReceiver<TestMetadata>) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut model = Model::new(recv);

    while model.mode() != AppState::Done {
        if let Some(msg) = model.handle_event().await? {
            model.update(msg).await?;
        }

        terminal.draw(|f| model.view(f))?;
    }
    ratatui::restore();
    Ok(())
}
