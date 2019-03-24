use std::net::ToSocketAddrs;
use std::sync::{Arc, RwLock};
use std::vec::Vec;
use std::string::String;
use std::iter::IntoIterator;

extern crate rustls;
use crate::nts_ke::server::rustls::Session;
use rustls::TLSError;

use tokio_rustls::{
    rustls::{NoClientAuth, ServerConfig},
    TlsAcceptor,
};

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::{AsyncRead, AsyncWrite, Future, Stream, Sink, stream};
use tokio::codec::Framed;

use crate::config::parse_nts_ke_config;

use crate::cookie;
use crate::cookie::NTSKeys;
use super::protocol;

fn gen_key_from_channel<T: AsyncRead + AsyncWrite>(
    stream: tokio_rustls::server::TlsStream<T>,
) -> (tokio_rustls::server::TlsStream<T>, NTSKeys) {
    let (_underlying, server_session) = stream.get_ref();
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
    let label = "EXPORTER-nts/1".as_bytes();
    session.export_keying_material(&mut keys.c2s, label, context_c2s)?;
    session.export_keying_material(&mut keys.s2c, label, context_s2c)?;

    Ok(keys)
}

fn response (keys: NTSKeys, master_key: Arc<RwLock<Vec<u8>>>) -> Vec<protocol::NtsKeRecord> {
    let actual_key = master_key.read().unwrap();
    let cookie = cookie::make_cookie(keys, &actual_key);
    let mut response: Vec<protocol::NtsKeRecord> = Vec::new();
    let mut aead_rec = protocol::NtsKeRecord {
        critical: false,
        record_type: 4,
        contents: vec![0, 15],
    };
    let mut end_rec = protocol::NtsKeRecord {
        critical: true,
        record_type: 0,
        contents: vec![],
    };
    let mut cookie_rec = protocol::NtsKeRecord{
        critical: false,
        record_type: 5,
        contents: cookie,
    };

    response.push(aead_rec);
    response.push(cookie_rec);
    response.push(end_rec);
    response
}

pub fn start_nts_ke_server(config_filename: &str) {
    // First parse config for TLS server using local config module.
    let parsed_config = parse_nts_ke_config(config_filename);
    let master_key = parsed_config.cookie_key;
    let real_key = Arc::new(RwLock::new(master_key));
    let mut server_config = ServerConfig::new(NoClientAuth::new());
    let alpn_proto = String::from("ntske/1");
    let alpn_bytes = alpn_proto.into_bytes();
    server_config
        .set_single_cert(parsed_config.tls_certs, parsed_config.tls_keys[0].clone())
        .expect("invalid key or certificate");
    server_config.set_protocols(&[alpn_bytes]);
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
        let real_key = real_key.clone();
        let done = config
            .accept(conn)
            .map(|socket | gen_key_from_channel(socket))
            .and_then(|(socket, key)| {
                let proto_sock = Framed::new(socket, protocol::NtsKeCodec{});
                let resp = response(key, real_key);
                let source_iter: std::vec::IntoIter<protocol::NtsKeRecord>  = resp.into_iter();
                let answer_str = stream::iter_ok::<std::vec::IntoIter<protocol::NtsKeRecord>, std::io::Error>(source_iter);
                proto_sock.send_all(answer_str)
            })
            .map( move |_| println!("Successful connection!"))
            .map_err(move |err| println!("Error: {:?}-{:?}", err, addr));
        tokio::spawn(done);
        Ok(())
    });

    // Run TLS server.
    println!("Starting NTS-KE server over TCP/TLS on {:?}", addr);
    tokio::run(done.map_err(drop))
}
