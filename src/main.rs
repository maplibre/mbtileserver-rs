extern crate clap;
extern crate flate2;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate regex;
extern crate serde;
extern crate serde_json;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::process;

mod config;
mod errors;
mod service;
mod tiles;
mod utils;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args = match config::parse() {
        Ok(args) => args,
        Err(err) => {
            println!("{}", err);
            process::exit(1)
        }
    };

    println!("Serving tiles from {}", args.directory.display());

    let tilesets = tiles::discover_tilesets(String::new(), args.directory);

    let addr = ([0, 0, 0, 0], args.port).into();

    let make_service = make_service_fn(move |_conn| {
        let tilesets = tilesets.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                service::get_service(req, tilesets.clone())
            }))
        }
    });

    let server = match Server::try_bind(&addr) {
        Ok(builder) => builder.serve(make_service),
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
