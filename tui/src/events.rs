use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use tokio::sync::mpsc;

pub enum IncomingEvents {
    InputRequest(String),
}

pub enum OutgoingEvents {
    OperatorInput(String),
}

pub struct EventHandler {
    pub receiver_in: mpsc::UnboundedReceiver<IncomingEvents>,
    pub sender_in: mpsc::UnboundedSender<IncomingEvents>,
    pub receiver_out: mpsc::UnboundedReceiver<OutgoingEvents>,
    pub sender_out: mpsc::UnboundedSender<OutgoingEvents>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender_in, receiver_in) = mpsc::unbounded_channel();
        let (sender_out, receiver_out) = mpsc::unbounded_channel();
        Self {
            receiver_in,
            sender_in,
            receiver_out,
            sender_out,
        }
    }
}
