extern crate clap;
extern crate flate2;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod config;
mod errors;
mod server;
mod service;
mod tiles;
mod utils;

fn main() {
    pretty_env_logger::init_timed();

    let args = match config::parse(config::get_app().get_matches()) {
        Ok(args) => args,
        Err(err) => {
            error!("{}", err);
            std::process::exit(1)
        }
    };

    if let Err(e) = server::run(args) {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
