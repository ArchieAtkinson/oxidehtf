mod app;
mod events;

use tokio::time::Duration;

use app::{handle_event, update, view, AppState, Model};
use color_eyre::eyre::Result;
use events::{EventHandler, IncomingEvents, OutgoingEvents};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let mut events = EventHandler::new();
    let mut handle =
        tokio::spawn(async move { app(events.receiver_out, events.sender_in.clone()).await });

    loop {
        tokio::select! {
            Some(data) = events.receiver_in.recv() => {
                match data {
                    IncomingEvents::InputRequest(s) => {
                        let out = OutgoingEvents::OperatorInput(s);
                        events.sender_out.send(out)?;
                    }
                }
            }
            _ = &mut handle => {
                break;
            }


        }
    }
    Ok(())
}

async fn app(
    recv: mpsc::UnboundedReceiver<OutgoingEvents>,
    send: mpsc::UnboundedSender<IncomingEvents>,
) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut model = Model::new(recv, send);

    while model.mode() != AppState::Done {
        terminal.draw(|f| view(&model, f))?;

        if let Some(msg) = handle_event(&model).await? {
            update(&mut model, msg).await?;
        }
    }
    ratatui::restore();
    Ok(())
}
