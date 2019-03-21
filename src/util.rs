use std::net::{SocketAddr, ToSocketAddrs};

pub fn get_socket_addr_from_str(addr_str: &str) -> SocketAddr {
  let addr_socket = addr_str.to_socket_addrs()
    .expect("Could not turn addr into socket address")
    .next()
    .expect("Could not iterate to the next value of the addr Iterator given by to_socket_addrs");
  addr_socket
}