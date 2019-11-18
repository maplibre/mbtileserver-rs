extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate pretty_env_logger;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate tera;

use hyper::rt::{self, Future};
use hyper::service::service_fn_ok;
use hyper::Server;

mod config;
mod service;
mod tiles;

fn main() {
    pretty_env_logger::init();

    let args = config::parse();

    println!("Serving tiles from {}", args.directory.display());

    let addr = ([127, 0, 0, 1], args.port).into();

    let server = Server::bind(&addr)
        .serve(move || {
            // This is the `Service` that will handle the connection.
            // `service_fn_ok` is a helper to convert a function that
            // returns a Response into a `Service`.
            service_fn_ok(service::get_service(args.directory.clone()))
        })
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);

    rt::run(server);
}
