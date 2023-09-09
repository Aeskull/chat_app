use crate::message::Message;
use crate::prelude::*;
use openssl::symm::{Cipher, encrypt, decrypt};
use openssl::pkey::Public;
use openssl::rsa::{Padding, Rsa};
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
    const RSA_SIZE: u32 = 2048;
    const SYMM_SIZE: usize = 32;

    // Make the connection to the server
    // let mut conn = TcpStream::connect(ip).await?;
    let mut stream = match TcpStream::connect(ip).await {
        Ok(conn) => conn,
        Err(_e) => {
            // eprintln!("ERROR: {e}\nDefaulting to '127.0.0.1:8080'");
            TcpStream::connect("127.0.0.1:42530").await?
        }
    };

    let cl_rsa = Rsa::generate(RSA_SIZE)?;
    let ciph = Cipher::aes_256_cbc();
    let iv = gen_rand_iv(ciph.block_size());
    let mut sv_pub_key: Option<Rsa<Public>> = None;

    {
        let pub_key = cl_rsa.public_key_to_der()?;
        let key_len = (pub_key.len() as u32).to_be_bytes();
        let header = "PUB".as_bytes();
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
                        let key_len = u32::from_be_bytes(len_buf) as usize; // Parse to usize.
                        let typ = String::from_utf8_lossy(&key_buf).to_string(); // Get the type of the packet.
                        match typ.as_str() {
                            "PUB" => {
                                let mut key = vec![0u8; key_len];
                                if stream.read_exact(&mut key).await.is_err() {
                                    break;
                                }
                                sv_pub_key = Some(Rsa::public_key_from_der(&key)?);
                            },
                            "ENC" => {
                                if sv_pub_key.is_some() {
                                    let mut key = vec![0u8; key_len];
                                    stream.read_exact(&mut key).await?;

                                    let mut msg_hed = [0u8; 3];
                                    stream.read_exact(&mut msg_hed).await?;
                                    if &msg_hed == b"MSG" {
                                        let mut msg_len = [0u8; 4];
                                        stream.read_exact(&mut msg_len).await?;
                                        let len = u32::from_be_bytes(msg_len) as usize;
                                        let mut msg = vec![0u8; len];
                                        stream.read_exact(&mut msg).await?;

                                        let dec_key = {
                                            let mut t = vec![0u8; RSA_SIZE as usize];
                                            let len = cl_rsa.private_decrypt(&key, &mut t, Padding::PKCS1)?;
                                            t[0..len].to_owned()
                                        };

                                        decrypt(ciph, &dec_key, Some(&iv), &msg)?;
                                        let msg_str = String::from_utf8_lossy(&msg);
                                        ssx.send(msg_str.to_string()).await?;
                                    } else {
                                        stream.shutdown().await?;
                                        eprintln!("An error has occured");
                                        return Ok(());
                                    }
                                    
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
                    if let Some(pub_rsa) = &sv_pub_key {
                        let msg = Message::new(&user, &m);      // Create the message struct.
                        let msg_bytes = {
                            let t = json!(msg).to_string();
                            t.as_bytes().to_vec()
                        };
                        let key = gen_rand_symm(SYMM_SIZE);

                        let msg_enc = encrypt(ciph, &key, Some(&iv), &msg_bytes)?;
                        let key_enc = {
                            let mut t = vec![0u8; RSA_SIZE as usize];
                            let len = pub_rsa.public_encrypt(&key, &mut t, Padding::PKCS1)?;
                            t[0..len].to_owned()
                        };

                        let msg_len = (msg_enc.len() as u32).to_be_bytes(); // Write the length header.
                        let key_len = (key_enc.len() as u32).to_be_bytes();
                        let key_header = "ENC".as_bytes();
                        let msg_header = "MSG".as_bytes();

                        stream.write_all(&[key_header, &key_len, &key_enc, msg_header, &msg_len, &msg_enc].concat()).await?; // Write the header and the json to the connection.
                        stream.flush().await?; // Flush the connection buffer.
                    }
                }
            }
        }
    }
    Ok(())
}

fn gen_rand_symm(prec: usize) -> Vec<u8> {
    let mut key = vec![0u8; prec];
    openssl::rand::rand_bytes(&mut key).unwrap();
    key
}

fn gen_rand_iv(block_size: usize) -> Vec<u8> {
    let mut key = vec![0u8; block_size];
    openssl::rand::rand_bytes(&mut key).unwrap();
    key
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
