use crate::{prelude::*, message::Message};

/// ### The main reciever function
/// 
/// It will be listening for incoming messages. If one is found, it will parse it and display it.
pub async fn reciever_loop(tx: tokio::sync::mpsc::Sender<Message>) -> Result<()> {
    
    Ok(())
}

#[cfg(test)]
mod tests {
    
}