use std::sync::Arc;
use std::net::ToSocketAddrs;

use tokio_rustls::{
  rustls::{
      internal::pemfile::{certs, pkcs8_private_keys},
      NoClientAuth, ServerConfig,
  },
  TlsAcceptor,
};

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::{Future, Stream};

use crate::config::parse_nts_ke_config;

pub fn start_nts_ke_server(config_filename: &str) {
  let mut parsedConfig = parse_nts_ke_config(config_filename);
  println!("{:?}", parsedConfig);

  let mut serverConfig = ServerConfig::new(NoClientAuth::new());
  serverConfig
    .set_single_cert(parsedConfig.tls_certs, parsedConfig.tls_keys.remove(0))
    .expect("invalid key or certificate");
  let config = TlsAcceptor::from(Arc::new(serverConfig));

  let addr = parsedConfig.addr
    .to_socket_addrs()
    .unwrap()
    .next()
    .unwrap();
  let socket = TcpListener::bind(&addr).unwrap();
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

  tokio::run(done.map_err(drop))
}