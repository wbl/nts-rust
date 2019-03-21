use std::net::ToSocketAddrs;
use std::sync::Arc;

extern crate rustls;
use crate::nts_ke::server::rustls::Session;
use rustls::TLSError;
use tokio_rustls::server::TlsStream;

use tokio_rustls::{
    rustls::{NoClientAuth, ServerConfig},
    TlsAcceptor,
};

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::{AsyncRead, AsyncWrite, Future, Stream};

use crate::config::parse_nts_ke_config;

fn gen_key_from_channel<T: AsyncRead + AsyncWrite>(
    stream: tokio_rustls::server::TlsStream<T>,
) -> tokio_rustls::server::TlsStream<T> {
    let (inner, server_session) = stream.get_ref();
    gen_key(server_session).expect("Failure to generate keys");
    return stream;
}

fn gen_key(session: &rustls::ServerSession) -> Result<(), TLSError> {
    let mut c2s_key = [0; 32];
    let mut s2c_key = [0; 32];
    let c2s_con = [0, 0, 0, 15, 00];
    let s2c_con = [0, 0, 0, 15, 01];
    let context_c2s = Some(&c2s_con[..]);
    let context_s2c = Some(&s2c_con[..]);
    let label = "EXPORTER-network-time-security/1".as_bytes();
    session.export_keying_material(&mut c2s_key, label, context_c2s)?;
    session.export_keying_material(&mut s2c_key, label, context_s2c)?;
    println!("c2s_key: {:?}", c2s_key);
    println!("s2c_key: {:?}", s2c_key);
    Ok({})
}

pub fn start_nts_ke_server(config_filename: &str) {
    // First parse config for TLS server using local config module.
    let mut parsed_config = parse_nts_ke_config(config_filename);

    let mut server_config = ServerConfig::new(NoClientAuth::new());
    server_config
        .set_single_cert(parsed_config.tls_certs, parsed_config.tls_keys.remove(0))
        .expect("invalid key or certificate");
    let config = TlsAcceptor::from(Arc::new(server_config));

    let addr = parsed_config
        .addr
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
            .map(gen_key_from_channel)
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
