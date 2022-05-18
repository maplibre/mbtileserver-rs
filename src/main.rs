use clap::Parser;
use log::error;

mod config;
mod errors;
mod server;
mod service;
mod tiles;
mod utils;

fn main() {
    pretty_env_logger::init_timed();

    let args = config::Args::parse().post_parse().unwrap_or_else(|err| {
        error!("{err}");
        std::process::exit(1)
    });

    if let Err(e) = server::run(args) {
        error!("Server error: {e}");
        std::process::exit(1);
    }
}
