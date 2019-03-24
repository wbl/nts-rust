use tokio::io;
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::boxed::Box;

use std::result;
use std::error;

use super::protocol;

pub fn run_nts_ke_client(addr: &str) -> Result<(), Box<std::error::Error>> {
    let parsed_addr = addr.parse::<SocketAddr>()?;
    let socket = TcpStream::connect(&parsed_addr);
    let mut config = rustls::ClientConfig::new();
    let alpn_proto = String::from("ntske/1");
    let alpn_bytes = alpn_proto.into_bytes();
    config.set_protocols(&[alpn_bytes]);
    
    
    Ok(())
}
