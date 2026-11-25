use anyhow::Result;
use my_http::tcp::{Client, Listener};
use std::net::SocketAddr;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let directory = &args[2];

    let listener_config = Listener::new(SocketAddr::from(([127, 0, 0, 1], 4222))).unwrap();
    let tcp_listener = listener_config.listen().unwrap();

    Client::handle_clients(tcp_listener, directory)?;

    Ok(())
}
