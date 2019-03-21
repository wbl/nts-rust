use std::error::Error;

#[derive(Debug, Copy, Clone)]
struct NTSKeys {
    c2s: [u8; 32],
    s2c: [u8; 32],
}

fn make_cookie(keys: NTSKeys, cookie_key: Vec<u8>) -> Vec<u8> {}

fn decode_cookie(cookie: Vec<u8>, cookie_key: Vec<u8>) -> Result<NTSKeys, Box<Error>> {}
