use my_http::tcp::{
  Client,
  Listener
};
use std::net::SocketAddr;

// https://www.rfc-editor.org/info/rfc7230/#section-3.1.1
fn main() {
  let socket_addresses: Vec<SocketAddr> = vec![
    SocketAddr::from(([127, 0, 0, 1], 4221)),
    SocketAddr::from(([127, 0, 0, 1], 4222)),
  ];

  let listener_config = Listener::new(socket_addresses).unwrap();
  let tcp_listener = listener_config.listen().unwrap();
  
  Client::handle_clients(tcp_listener);
}
