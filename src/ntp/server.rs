use tokio::net::UdpSocket;
use tokio::prelude::*;

use std::net::SocketAddr;
use std::io;

use crate::config::parse_ntp_config;
use crate::util::get_socket_addr_from_str;

// TODO: perhaps https://github.com/tokio-rs/tokio/blob/master/examples/udp-codec.rs would be a good example to look at
// for implementing a server that can handle a custom protocol.

struct Server {
  socket: UdpSocket,
  buf: Vec<u8>,
  to_send: Option<(usize, SocketAddr)>,
}

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            if let Some((size, peer)) = self.to_send {
                let amt = try_ready!(self.socket.poll_send_to(&self.buf[..size], &peer));
                println!("Echoed {}/{} bytes to {}", amt, size, peer);
                self.to_send = None;
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            self.to_send = Some(try_ready!(self.socket.poll_recv_from(&mut self.buf)));
        }
    }
}

pub fn start_ntp_server(config_filename: &str) -> Result<(), Box<std::error::Error>> {
  // First parse config for TLS server using local config module.
  let parsed_config = match parse_ntp_config(config_filename) {
    Ok(c)  => { c }
    Err(e) => { panic!(e.to_string()) }
  };

  let addr = get_socket_addr_from_str(&parsed_config.addr);

  let socket = UdpSocket::bind(&addr)?;
  println!("UDP NTP server listening on: {}", socket.local_addr()?);

  let server = Server {
        socket: socket,
        buf: vec![0; 1024],
        to_send: None,
    };

  tokio::run(server.map_err(|e| println!("server error = {:?}", e)));
  Ok(())
}