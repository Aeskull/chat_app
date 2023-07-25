use crate::message::Message;
use crate::prelude::*;
use std::io::{stdin, stdout, Write};
use std::sync::OnceLock;

/// Macro to get the input in a concise manor.
macro_rules! get {
    ($x: expr) => {{
        print!("> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut $x)
    }};
}

/// ### The main Sender loop.
///
/// Loops ad infinitum. It will handle input, parsing of input, and passing the data along to be sent.
pub async fn sender_loop(mut tx: tokio::sync::mpsc::Receiver<Message>) -> Result<()> {
    while let Some(m) = tx.recv().await {
        println!("{m:?}");
    }
    Ok(())
}
