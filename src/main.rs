#[macro_use] extern crate futures;
extern crate tokio;

mod config;
mod nts_ke;
mod ntp;
mod util;

use clap::App;
use clap::Arg;
use clap::SubCommand;

use crate::nts_ke::server::start_nts_ke_server;
use crate::ntp::server::start_ntp_server;

fn app() -> App<'static, 'static> {
  App::new("cf-nts")
    .about("cloudflare's NTS implementation.")
    .version("v0.1")
    // .subcommand_required_else_help(true) TODO: this seems to be very broken in the clap crate.
    .subcommands( vec![
      SubCommand::with_name("nts-ke").about("Runs NTS-KE server over TLS/TCP")
        .arg(Arg::with_name("config_file").index(1).required(true)),
      SubCommand::with_name("ntp").about("Interfaces with NTP using UDP")
        .arg(Arg::with_name("config_file").index(1).required(true)),
    ])
}

fn main() {
  let matches = app().get_matches();

  // TODO: remove this if statement when .subcommand_required_else_help(true) works.
  if let None = matches.subcommand {
    println!("You must specify a subcommand, nts-ke or ntp.");
    return;
  }

  if let Some(nts_ke) = matches.subcommand_matches("nts-ke") {
    // We can use unwrap because config_file is already checked for inclusion by the App struct (cannot be non-empty)
    let config_file = nts_ke.value_of("config_file").unwrap(); 
    start_nts_ke_server(config_file);
  }

  if let Some(ntp) = matches.subcommand_matches("ntp") {
    // Same logic for using unwrap() above applies here.
    let config_file = ntp.value_of("config_file").unwrap();
    if let Err(err) = start_ntp_server(config_file) {
      println!("Starting UDP server failed: {}", err);
    }
  }
}
