use std::sync::OnceLock;

use color_eyre::eyre::{eyre, Result};
use tokio::sync::{mpsc, Mutex};

// Global Singleton to request input from the operator
// by providing the prompt to show to the operator
//
// This module "owns" the channel for receiving the input and is given
// the sender for the prompt channel which is "owned" by the UI
//
// channel (OperatorPrompt) - test_to_operator
//         - sender is used by test
//         - recv is used by TUI (owner)
//
// channel (OperatorInput) - operator_to_test
//         - sender is used by tui
//         - revc is used by test (owner)

static OPERATOR_COMMS: OnceLock<Mutex<TestOperatorComms>> = OnceLock::new();

#[derive(Debug)]
pub struct OperatorPrompt(pub String);

#[derive(Debug)]
pub struct OperatorInput(pub String);

struct TestOperatorComms {
    prompt_sender: mpsc::UnboundedSender<OperatorPrompt>,
    operator_recivier: mpsc::UnboundedReceiver<OperatorInput>,
}

pub struct UIOperatorComms {
    pub prompt_receiver: mpsc::UnboundedReceiver<OperatorPrompt>,
    pub operator_sender: mpsc::UnboundedSender<OperatorInput>,
}
pub fn init() -> Result<UIOperatorComms> {
    let (operator_input_tx, operator_input_rx) = mpsc::unbounded_channel::<OperatorInput>();
    let (operator_prompt_tx, operator_prompt_rx) = mpsc::unbounded_channel::<OperatorPrompt>();
    OPERATOR_COMMS
        .set(Mutex::new(TestOperatorComms {
            prompt_sender: operator_prompt_tx,
            operator_recivier: operator_input_rx,
        }))
        .map_err(|_| eyre!("Failed to init Operator Comms"))?;

    Ok(UIOperatorComms {
        prompt_receiver: operator_prompt_rx,
        operator_sender: operator_input_tx,
    })
}

pub fn request_input(prompt: impl Into<String>) -> Result<String> {
    let mut comms = OPERATOR_COMMS
        .get()
        .expect("Failed to get oncelock")
        .blocking_lock();
    comms.prompt_sender.send(OperatorPrompt(prompt.into()))?;
    let OperatorInput(input) = comms
        .operator_recivier
        .blocking_recv()
        .ok_or(eyre!("Failed to get input"))?;
    Ok(input)
}
