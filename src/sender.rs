use crate::message::Message;
use crate::prelude::*;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// ### The main Sender loop.
///
/// Loops ad infinitum. It will handle input, parsing of input, and recieving data to be sent to the reciever.
pub async fn sender_loop(
    mut tx: tokio::sync::mpsc::Receiver<String>,
    user: String,
    ip: String,
    ssx: tokio::sync::mpsc::Sender<String>,
) -> Result<()> {
    // Make the connection to the server
    // let mut conn = TcpStream::connect(ip).await?;
    let mut conn = match TcpStream::connect(ip).await {
        Ok(conn) => conn,
        Err(_e) => {
            // eprintln!("ERROR: {e}\nDefaulting to '127.0.0.1:8080'");
            TcpStream::connect("127.0.0.1:8080").await?
        }
    };

    // Main loop
    loop {
        let mut len_buf = [0u8; 4]; // Length buffer at the beginning of the package.

        // Check for either an incoming package to be sent to the server, or a package from the server.
        tokio::select! {
            result = conn.read_exact(&mut len_buf) => { // Check for message from server.
                match result {
                    Ok(0) => break, // Break if the connection was closed.
                    Ok(_) => {
                        // Get the length from the package header
                        let msg_len = u32::from_be_bytes(len_buf) as usize;
                        let mut msg_buf = vec![0u8; msg_len]; // Allocate a vector from the package's header.
                        if conn.read_exact(&mut msg_buf).await.is_err() {
                            break;
                        }
                        let msg = String::from_utf8_lossy(&msg_buf).to_string(); // Convert the package to a string.
                        ssx.send(msg).await?; // Send to the reciever.
                    },
                    Err(e) => {
                        eprintln!("Error reading from client: {e:?}");
                        break; // Break on error.
                    }
                }
            },
            Some(m) = tx.recv() => { // Check for message from the terminal.
                if !m.is_empty() {
                    let msg = Message::new(&user, &m);      // Create the message struct.
                    let msg_json = json!(msg).to_string();  // Parse it to a json.

                    let msg_len = (msg_json.len() as u32).to_be_bytes(); // Write the length header.

                    conn.write_all(&[&msg_len, msg_json.as_bytes()].concat()).await?; // Write the header and the json to the connection.
                    conn.flush().await?; // Flush the connection buffer.
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
