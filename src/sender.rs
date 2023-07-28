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
    loop {
        let mut len_buf = [0u8; 4];
        tokio::select! {
            result = conn.read_exact(&mut len_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        let msg_len = u32::from_be_bytes(len_buf) as usize;
                        let mut msg_buf = vec![0u8; msg_len];
                        if conn.read_exact(&mut msg_buf).await.is_err() {
                            break;
                        }
                        let msg = String::from_utf8_lossy(&msg_buf).to_string();
                        ssx.send(msg).await?;
                    },
                    Err(e) => {
                        eprintln!("Error reading from client: {e:?}");
                        break;
                    }
                }
            },
            Some(m) = tx.recv() => {
                if m.len() > 0 {
                    let msg = Message::new(&user, &m);
                    let msg_json = json!(msg).to_string();

                    let msg_len = (msg_json.len() as u32).to_be_bytes();

                    conn.write_all(&[&msg_len, msg_json.as_bytes()].concat()).await?;
                    conn.flush().await?;
                }
            }
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
