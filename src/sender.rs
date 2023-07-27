use crate::message::Message;
use crate::prelude::*;
use serde_json::json;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use std::fs::OpenOptions;
use std::io::Write;

/// ### The main Sender loop.
///
/// Loops ad infinitum. It will handle input, parsing of input, and passing the data along to be sent.
pub async fn sender_loop(
    mut tx: tokio::sync::mpsc::Receiver<String>,
    user: String,
    ip: String,
    ssx: tokio::sync::mpsc::Sender<String>,
) -> Result<()> {
    let mut f = OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open("log_s.txt")?;

    let mut conn = TcpStream::connect(ip).await?;
    let mut buf = [0u8; 1024];
    loop {
        match tx.recv().await {
            Some(m) if m.len() > 0 => {
                let msg = Message::new(&user, &m);
                let msg_json = json!(msg).to_string();

                conn.write_all(msg_json.as_bytes()).await?;
            },
            _ => {},
        }

        tokio::select! {
            result = conn.read(&mut buf) => {
                match result {
                    Ok(0) => break,
                    Ok(bytes_read) => {
                        let chunk = String::from_utf8_lossy(&buf[0..bytes_read]);
                        writeln!(f, "Sending to reciever: {chunk}")?;
                        ssx.send(chunk.to_string()).await?;
                    },
                    Err(e) => {
                        eprintln!("Error reading from client: {e:?}");
                        break;
                    }
                }
            },
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {},
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn binding_test() {
        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
        let conn = TcpStream::connect("127.0.0.1:8080").await;
        _ = listener.accept().await.unwrap();
        println!("{conn:?}");
    }
}
