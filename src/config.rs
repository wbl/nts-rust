use config::Config;
use tokio_rustls::{
  rustls::{
    internal::pemfile::{certs, pkcs8_private_keys},
    Certificate, PrivateKey,
  },
};

use std::io::BufReader;
use std::fs;

#[derive(Debug)]
pub struct ConfigNTSKE {
  pub tls_certs: Vec<Certificate>,
  pub tls_keys: Vec<PrivateKey>,
  pub cookie_key: Vec<u8>,
  pub addr: String,
}

#[derive(Debug)]
pub struct ConfigNTP {
  pub addr: String,
  pub cookie_key: Vec<u8>
}

fn load_tls_certs(path: String) -> Vec<Certificate> {
  let cert_file = fs::File::open(path).expect("Could not open tls certs file");
  let buf = &mut BufReader::new(cert_file);
  certs(buf).expect("Could not parse certificates")
}

fn load_tls_keys(path: String) -> Vec<PrivateKey> {
  let cert_file = fs::File::open(path).expect("Could not open tls keys file");
  let buf = &mut BufReader::new(cert_file);
  pkcs8_private_keys(buf).expect("Could not parse keys")
}

fn load_cookie_key(path: String) -> Vec<u8> {
  fs::read(path).expect("Unable to read file")
}

pub fn parse_nts_ke_config(config_filename: &str) -> Result<ConfigNTSKE, Box<std::error::Error>> {
  let mut settings = Config::default();
  settings.merge(config::File::with_name(config_filename)).expect("Could not parse yaml");

  // All config filenames MUST be given with relative paths to where the server is run.
  // Or else cf-nts will try to open the file while in the incorrect directory.
  let tls_cert_filename = settings.get_str("tls_cert_file").expect("tls_cert_file required in yaml");
  let tls_key_filename = settings.get_str("tls_key_file").expect("tls_key_file required in yaml");
  let cookie_key_filename = settings.get_str("cookie_key_file").expect("cookie_key_file required in yaml");
  let addr = settings.get_str("addr").expect("addr required in yaml");
  
  let config = ConfigNTSKE {
    tls_certs: load_tls_certs(tls_cert_filename),
    tls_keys: load_tls_keys(tls_key_filename),
    cookie_key: load_cookie_key(cookie_key_filename),
    addr: addr
  };
  Ok(config)
}

pub fn parse_ntp_config(config_filename: &str) -> Result<ConfigNTP, Box<std::error::Error>> {
  let mut settings = Config::default();
  settings.merge(config::File::with_name(config_filename)).expect("Could not parse yaml");

  // All config filenames MUST be given with relative paths to where the server is run.
  // Or else cf-nts will try to open the file while in the incorrect directory.
  let cookie_key_filename = settings.get_str("cookie_key_file").expect("cookie_key_file required in yaml");
  let addr = settings.get_str("addr").expect("addr required in yaml");
  
  let config = ConfigNTP {
    cookie_key: load_cookie_key(cookie_key_filename),
    addr: addr,
  };
  Ok(config)
}