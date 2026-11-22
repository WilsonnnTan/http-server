use my_http::tcp::{
  Client,
  Listener
};
use std::net::SocketAddr;
use anyhow::Result;

// https://www.rfc-editor.org/info/rfc7230/#section-3.1.1
fn main() -> Result<()>{
  let listener_config = Listener::new(SocketAddr::from(([127, 0, 0, 1], 4222))).unwrap();
  let tcp_listener = listener_config.listen().unwrap();
  
  Client::handle_clients(tcp_listener)?;

  Ok(())
}
