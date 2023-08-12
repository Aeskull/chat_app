use crate::message::Message;
use crate::prelude::*;
use openssl::pkey::Public;
use openssl::rsa::{Padding, Rsa};
use serde_json::json;
use std::sync::OnceLock;
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
    let mut stream = match TcpStream::connect(ip).await {
        Ok(conn) => conn,
        Err(_e) => {
            // eprintln!("ERROR: {e}\nDefaulting to '127.0.0.1:8080'");
            TcpStream::connect("127.0.0.1:42530").await?
        }
    };

    let cl_rsa = Rsa::generate(2048)?;
    let sv_pub_key = OnceLock::<Rsa<Public>>::new();

    {
        let pub_key = cl_rsa.public_key_to_der()?;
        let key_len = (pub_key.len() as u32).to_be_bytes();
        let header = "KEY".as_bytes();
        stream
            .write_all(&[header, &key_len, &pub_key].concat())
            .await?;
    }

    // Main loop
    loop {
        let mut key_buf = [0u8; 3]; // Buffer to determine type of message.
        let mut len_buf = [0u8; 4]; // Length buffer at the beginning of the packet.

        // Check for either an incoming packet to be sent to the server, or a packet from the server.
        tokio::select! {
            result = stream.read_exact(&mut key_buf) => { // Check for message from server.
                match result {
                    Ok(0) => break, // Break if the connection was closed.
                    Ok(_) => {
                        stream.read_exact(&mut len_buf).await?; // Get the length
                        let msg_len = u32::from_be_bytes(len_buf) as usize; // Parse to usize.
                        let typ = String::from_utf8_lossy(&key_buf).to_string(); // Get the type of the packet.
                        match typ.as_str() {
                            "KEY" => {
                                let mut key = vec![0u8; msg_len];
                                if stream.read_exact(&mut key).await.is_err() {
                                    break;
                                }
                                sv_pub_key.set(Rsa::public_key_from_der(&key)?).unwrap();
                            },
                            "MSG" => {
                                if sv_pub_key.get().is_some() {
                                    let mut msg_buf = vec![0u8; msg_len]; // Allocate a vector from the packet's header.
                                    if stream.read_exact(&mut msg_buf).await.is_err() {
                                        break;
                                    }
                                    let mut msg: Vec<u8> = vec![];
                                    cl_rsa.private_decrypt(&msg_buf, &mut msg, Padding::PKCS1)?;
                                    let msg = String::from_utf8_lossy(&msg).to_string(); // Convert the packet to a string.
                                    ssx.send(msg).await?; // Send to the reciever.
                                } else {
                                    eprintln!("No");
                                }
                            },
                            _ => return Ok(()),
                        }
                    },
                    Err(e) if e.kind() == tokio::io::ErrorKind::UnexpectedEof => {
                        ssx.send("Close".to_owned()).await?;
                        break; // Break on close message.
                    },
                    Err(e) => {
                        eprintln!("Error reading from server: {e:?}");
                        break; // Break on error.
                    }
                }
            },
            Some(m) = tx.recv() => { // Check for message from the terminal.
                if !m.is_empty() {
                    if let Some(pub_rsa) = sv_pub_key.get() {
                        let msg = Message::new(&user, &m);      // Create the message struct.
                        let msg_bytes = {
                            let t = json!(msg).to_string();
                            t.as_bytes().to_vec()
                        };
                        let mut msg_enc: Vec<u8> = vec![];
                        pub_rsa.public_encrypt(&msg_bytes, &mut msg_enc, Padding::PKCS1)?;

                        let msg_len = (msg_enc.len() as u32).to_be_bytes(); // Write the length header.
                        let header = "MSG".as_bytes();

                        stream.write_all(&[header, &msg_len, &msg_enc].concat()).await?; // Write the header and the json to the connection.
                        stream.flush().await?; // Flush the connection buffer.
                    }
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
