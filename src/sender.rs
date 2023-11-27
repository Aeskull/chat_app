use std::net::SocketAddr;

use {
    crate::{message::Message, prelude::ConnectionError},
    openssl::{
        pkey::Private,
        rsa::{Padding, Rsa},
        symm::{decrypt, encrypt, Cipher},
    },
    serde_json::json,
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    },
};

/// ### The main Sender loop.
///
/// Loops ad infinitum. It will handle input, parsing of input, and recieving data to be sent to the reciever.
pub async fn sender_loop(
    mut rx: tokio::sync::mpsc::Receiver<String>,
    stx: tokio::sync::mpsc::Sender<String>,
    user: String,
    ip: String,
) -> Result<(), ConnectionError> {
    const RSA_SIZE: u32 = 2048;
    const SYMM_SIZE: usize = 32;
    const DEFAULT_PORT: u16 = 42530;
    let mut first = true;

    // Make the connection to the server
    // Make the socket from an ip. Default to 127.0.0.1:42530 upon an invalid ip
    let mut sock = ip
        .parse::<SocketAddr>()
        .unwrap_or("127.0.0.1:42530".parse::<SocketAddr>().unwrap());
    let mut stream = match TcpStream::connect(sock).await {
        Ok(conn) => conn,
        Err(_) => {
            // If the connection failed, change the port to the default port and try again, returning if failing again.
            sock.set_port(DEFAULT_PORT);
            match TcpStream::connect(sock).await {
                Ok(t) => t,
                Err(_) => return Err(ConnectionError::new("connection refused")),
            }
        }
    };

    let cl_rsa = Rsa::generate(RSA_SIZE).unwrap();
    let ciph = Cipher::aes_256_cbc();
    let mut sv_prv_key: Option<Rsa<Private>> = None;

    {
        let pub_key = cl_rsa.public_key_to_der().unwrap();
        let key_len = (pub_key.len() as u32).to_be_bytes();
        let header = "PUB".as_bytes();
        stream
            .write_all(&[header, &key_len, &pub_key].concat())
            .await
            .unwrap();
    }

    // Main loop
    loop {
        let mut key_buf = [0u8; 3]; // Buffer to determine type of message.
        let mut len_buf = [0u8; 4]; // Length buffer at the beginning of the packet.

        // Check for either an incoming packet to be sent to the server, or a packet from the server.
        tokio::select! {
            result = stream.read_exact(&mut key_buf) => { // Check for message from server.
                match result {
                    Ok(0) => {
                        stx.send("C".to_owned()).await.unwrap(); // Close on connection terminated
                        break; // Break on close message.
                    },
                    Ok(_) => {
                        stream.read_exact(&mut len_buf).await.unwrap(); // Get the length
                        let key_len = u32::from_be_bytes(len_buf) as usize; // Parse to usize.
                        let typ = String::from_utf8_lossy(&key_buf).to_string(); // Get the type of the packet.
                        match typ.as_str() {
                            "PRV" if first => {
                                let mut key = vec![0u8; key_len];
                                stream.read_exact(&mut key).await.unwrap();
                                let symm = {
                                    let mut t = vec![0u8; RSA_SIZE as usize];
                                    let l = cl_rsa.private_decrypt(&key, &mut t, Padding::PKCS1).unwrap();
                                    t[0..l].to_owned()
                                };
                                let mut der_len_buf = [0u8; 4];
                                stream.read_exact(&mut der_len_buf).await.unwrap();
                                let der_len = u32::from_be_bytes(der_len_buf);
                                let mut der_enc = vec![0u8; der_len as usize];
                                stream.read_exact(&mut der_enc).await.unwrap();
                                let der = decrypt(ciph, &symm, None, &der_enc).unwrap();
                                sv_prv_key = Some(Rsa::private_key_from_der(&der).unwrap());

                                first = false;
                            },
                            "ENC" => {
                                if sv_prv_key.is_some() {
                                    let mut key = vec![0u8; key_len];
                                    stream.read_exact(&mut key).await.unwrap();

                                    let mut msg_len = [0u8; 4];
                                    stream.read_exact(&mut msg_len).await.unwrap();
                                    let len = u32::from_be_bytes(msg_len) as usize;

                                    let mut msg = vec![0u8; len];
                                    stream.read_exact(&mut msg).await.unwrap();

                                    let Some(dec_key) = sv_prv_key.as_mut() else {
                                        break;
                                    };

                                    let k = {
                                        let mut t = vec![0u8; RSA_SIZE as usize];
                                        let len = dec_key.private_decrypt(&key, &mut t, Padding::PKCS1).unwrap();
                                        t[0..len].to_owned()
                                    };

                                    let msg_str = decrypt(ciph, &k, None, &msg).unwrap();

                                    let msg_str = String::from_utf8_lossy(&msg_str);
                                    stx.send(msg_str.to_string()).await.unwrap();

                                } else {
                                    eprintln!("No");
                                }
                            },
                            _ => return Ok(()),
                        }
                    },
                    Err(e) if e.kind() == tokio::io::ErrorKind::UnexpectedEof => {
                        stx.send("C".to_owned()).await.unwrap();
                        break; // Break on close message.
                    },
                    Err(e) => {
                        eprintln!("Error reading from server: {e:?}");
                        break; // Break on error.
                    }
                }
            },
            Some(m) = rx.recv() => { // Check for message from the terminal.
                if !m.is_empty() {
                    if let Some(prv_rsa) = &sv_prv_key {
                        let msg = Message::new(&user, &m);      // Create the message struct.
                        let msg_bytes = {
                            let t = json!(msg).to_string();
                            t.as_bytes().to_vec()
                        };
                        let key = gen_rand_symm(SYMM_SIZE);

                        let msg_enc = encrypt(ciph, &key, None, &msg_bytes).unwrap();
                        let key_enc = {
                            let mut t = vec![0u8; RSA_SIZE as usize];
                            let len = prv_rsa.public_encrypt(&key, &mut t, Padding::PKCS1).unwrap();
                            t[0..len].to_owned()
                        };

                        let msg_len = (msg_enc.len() as u32).to_be_bytes(); // Write the length header.
                        let key_len = (key_enc.len() as u32).to_be_bytes();
                        let key_header = "ENC".as_bytes();

                        stream.write_all(&[key_header, &key_len, &key_enc, &msg_len, &msg_enc].concat()).await.unwrap(); // Write the header and the json to the connection.
                        stream.flush().await.unwrap(); // Flush the connection buffer.
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
