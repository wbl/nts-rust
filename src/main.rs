extern crate tokio;
extern crate tokio_rustls;

use clap::App;
use clap::Arg;
use std::fs::File;
use std::io::BufReader;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio_rustls::{
    rustls::{
        internal::pemfile::{certs, pkcs8_private_keys},
        Certificate, NoClientAuth, PrivateKey, ServerConfig,
    },
    TlsAcceptor,
};

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::{Future, Stream};

fn load_certs(path: &str) -> Vec<Certificate> {
    certs(&mut BufReader::new(File::open(path).unwrap())).unwrap()
}

fn load_keys(path: &str) -> Vec<PrivateKey> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path).unwrap())).unwrap()
}

fn app() -> App<'static, 'static> {
    App::new("nts-server")
        .about("cloudflare's nts implementation")
        .arg(Arg::with_name("addr").value_name("ADDR").required(true))
        .arg(
            Arg::with_name("cert")
                .value_name("FILE")
                .help("cert file.")
                .required(true),
        )
        .arg(
            Arg::with_name("key")
                .value_name("FILE")
                .help("key file, ECDSA should work")
                .required(true),
        )
}

fn main() {
    let matches = app().get_matches();
    let addr = matches
        .value_of("addr")
        .unwrap()
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let cert_file = matches.value_of("cert").unwrap();
    let key_file = matches.value_of("key").unwrap();

    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        .set_single_cert(load_certs(cert_file), load_keys(key_file).remove(0))
        .expect("invalid key or certificate");
    let config = TlsAcceptor::from(Arc::new(config));

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
