use crate::{message::Message, prelude::*};
use serde_json::from_str;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use std::io::Write;

/// ### The main reciever function
///
/// It will be listening for incoming messages. If one is found, it will parse it and display it.
pub async fn reciever_loop(tx: tokio::sync::mpsc::Sender<Message>, ip: String, mut srx: tokio::sync::mpsc::Receiver<String>) -> Result<()> {
    loop {
        match srx.recv().await {
            Some(m) => {
                let msg = from_str::<Message>(&m)?;
                let Ok(_) = tx.send(msg).await else {
                    return Ok(());
                };
            },
            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::message::Message;

    #[test]
    fn recieve_test() {
        let msg1 = Message::new("Aeskul", "Hello");
        let msg2 = Message::new("Akachi", "I hate you!");

        let json1 = json!(msg1).to_string();
        let json2 = json!(msg2).to_string();

        let combined = format!("{json1}{json2}");
        println!("{combined}");

        let objs = serde_json::to_value(combined).unwrap();
        let objs = objs.as_object();
        println!("{objs:?}");
    }
}
