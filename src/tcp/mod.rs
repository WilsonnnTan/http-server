use super::http_1_1::server::Server;
use anyhow::{Context, Result};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    sync::Arc,
    thread,
};

pub struct Listener {
    /// List of addresses to listen on
    socket_address: SocketAddr,
}

/// Implementation of listener for Tcp
impl Listener {
    pub fn new(socket_address: SocketAddr) -> Result<Self> {
        Ok(Self { socket_address })
    }

    pub fn listen(&self) -> Result<TcpListener> {
        TcpListener::bind(self.socket_address)
            .with_context(|| format!("failed to bind to {:?}", self.socket_address))
    }
}

pub struct Client;

impl Client {
    pub fn handle_clients(listener: TcpListener, directory: &str) -> Result<()> {
        let directory = Arc::new(directory.to_string());
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    println!("accepted new connection");

                    let dir = Arc::clone(&directory);
                    thread::spawn(move || {
                        let mut buf: [u8; 512] = [0; 512];
                        let mut request_buf: Vec<u8> = Vec::new();
                        loop {
                            let buf_size = stream.read(&mut buf).unwrap_or(0);
                            if buf_size == 0 {
                                break;
                            }
                            request_buf.extend(&buf[..buf_size]);
                            println!("accepted {} bytes", buf_size);

                            if request_buf.windows(4).any(|w| w == b"\r\n\r\n") {
                                break;
                            }
                        }

                        let server = Server::new();
                        stream
                            .write_all(
                                &server
                                    .handle_request(str::from_utf8(&request_buf).unwrap(), &dir)
                                    .unwrap(),
                            )
                            .unwrap();
                    });
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                }
            }
        }

        Ok(())
    }
}
