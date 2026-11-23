use my_http::tcp::{
  Client,
  Listener
};
use std::net::SocketAddr;
use anyhow::Result;


fn main() -> Result<()>{
  let listener_config = Listener::new(SocketAddr::from(([127, 0, 0, 1], 4222))).unwrap();
  let tcp_listener = listener_config.listen().unwrap();
  
  Client::handle_clients(tcp_listener)?;

  Ok(())
}
