use anyhow::{Context, Result, ensure};
use std::{io::{Write, Read}, net::{SocketAddr, TcpListener}};

pub struct Listener {
  /// List of addresses to listen on
  socket_addresses: Vec<SocketAddr>
}

/// Implementation of listener for Tcp
impl Listener {
  pub fn new(socket_addresses: Vec<SocketAddr>) -> Result<Self> {
    ensure!(!socket_addresses.is_empty(), "no socket addresses provided");

    Ok(
      Self {
        socket_addresses: socket_addresses
      }
    )
  }

  pub fn listen(&self) -> Result<TcpListener>{
    TcpListener::bind(self.socket_addresses.as_slice())
      .with_context(|| format!("failed to bind to {:?}", self.socket_addresses))
  }
}


pub struct Client;

impl Client {
  pub fn handle_clients(listener: TcpListener) {
    for stream in listener.incoming() {
      match stream {
        Ok(mut stream) => {
          println!("accepted new connection");

          let mut buf: [u8; 512] = [0; 512];
          let buf_size = stream.read(&mut buf).unwrap_or(0);
          println!("accepted {} bytes", buf_size);

          stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        },
        Err(e) => {
          eprintln!("error: {}", e);
        }
      }
    }
  }
}
