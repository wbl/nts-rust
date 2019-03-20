mod config;
mod nts_ke;
mod ntp;

use clap::App;
use clap::Arg;
use clap::SubCommand;

use crate::nts_ke::server::start_nts_ke_server;

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

  if let Some(_nts_ke) = matches.subcommand_matches("nts-ke") {
    let config_file = _nts_ke.value_of("config_file").unwrap();
    start_nts_ke_server(config_file);
  }

  if let Some(_ntp) = matches.subcommand_matches("ntp") {
    let config_file = matches.value_of("config_file").unwrap();
    println!("todo: finish implementing UDP server!");
  }
}
