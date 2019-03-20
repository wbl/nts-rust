use std::sync::Arc;
use std::net::ToSocketAddrs;

use tokio_rustls::{
  rustls::{
    NoClientAuth, ServerConfig,
  },
  TlsAcceptor,
};

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::{Future, Stream};

use crate::config::parse_nts_ke_config;

pub fn start_nts_ke_server(config_filename: &str) {
  // First parse config for TLS server using local config module.
  let mut parsed_config = parse_nts_ke_config(config_filename);

  let mut server_config = ServerConfig::new(NoClientAuth::new());
  server_config
    .set_single_cert(parsed_config.tls_certs, parsed_config.tls_keys.remove(0))
    .expect("invalid key or certificate");
  let config = TlsAcceptor::from(Arc::new(server_config));

  let addr = parsed_config.addr
    .to_socket_addrs()
    .unwrap()
    .next()
    .unwrap();

  let socket = TcpListener::bind(&addr).unwrap();

  // Now, actually setup TLS server behavior.
  let done = socket.incoming().for_each(move |conn| {
    let addr = conn.peer_addr().ok();
    let done = config
      .accept(conn)
      .and_then(|stream| {
        io::write_all(
          stream,
          &b"HTTP/1.0 200 ok\r\n\
          Connection: close\r\n\
          Content-length: 12\r\n\
          \r\n\
          Hello world!"[..],
        )
      })
      .and_then(|(stream, _)| io::flush(stream))
      .map(move |_| println!("Accept: {:?}", addr))
      .map_err(move |err| println!("Error: {:?}-{:?}", err, addr));
    tokio::spawn(done);
    Ok(())
  });

  // Run TLS server.
  println!("Starting NTS-KE server over TCP/TLS on {:?}", addr);
  tokio::run(done.map_err(drop))
}