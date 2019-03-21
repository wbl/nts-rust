use std::net::ToSocketAddrs;
use std::sync::Arc;

use std::vec::Vec;

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

use crate::cookie::NTSKeys;

struct NtsKeRecord {
    critical: bool,
    record_type: u16,
    contents: Vec<u8>,
}

fn serialize_record(rec: NtsKeRecord) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let our_type = 0;
    if critical {
        our_type = 1 << 15 + rec.record_type;
    } else {
        our_type = rec.record_type;
    }
    out.extend(our_type.to_be_bytes().iter().clone());
    let our_len = rec.contents.len() as u16;
    out.extend(our_len.to_be_bytes().iter().clone());
    out.append(contents);
    return out;
}

fn gen_key_from_channel<T: AsyncRead + AsyncWrite>(
    stream: tokio_rustls::server::TlsStream<T>,
) -> (tokio_rustls::server::TlsStream<T>, NTSKeys) {
    let (inner, server_session) = stream.get_ref();
    let res = gen_key(server_session).expect("Failure to generate keys");
    return (stream, res);
}

fn gen_key(session: &rustls::ServerSession) -> Result<NTSKeys, TLSError> {
    let mut keys: NTSKeys = NTSKeys {
        c2s: [0; 32],
        s2c: [0; 32],
    };
    let c2s_con = [0, 0, 0, 15, 00];
    let s2c_con = [0, 0, 0, 15, 01];
    let context_c2s = Some(&c2s_con[..]);
    let context_s2c = Some(&s2c_con[..]);
    let label = "EXPORTER-network-time-security/1".as_bytes();
    session.export_keying_material(&mut keys.c2s, label, context_c2s)?;
    session.export_keying_material(&mut keys.s2c, label, context_s2c)?;

    Ok(keys)
}

fn response(keys: NTSKeys, master_key: Vec<u8>) -> Vec<u8> {
    let cookie = cookie::make_cookie(keys, master_key);
    let mut response: Vec<u8> = Vec::new();
}
pub fn start_nts_ke_server(config_filename: &str) {
    // First parse config for TLS server using local config module.
    let mut parsed_config = parse_nts_ke_config(config_filename);
    let master_key = parsed_config.cookie_key;
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
            .and_then(|socket| {
                let buf: Vec<u8> = Vec::new();
                io::read_to_end(socket, buf)
            })
            .map(|(socket, buf)| gen_key_from_channel(socket))
            .and_then(|(socket, key)| io::write_all(socket, response(master_key, key)))
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
