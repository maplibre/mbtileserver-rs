extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate tera;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

mod config;
mod service;
mod tiles;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args = config::parse();

    println!("Serving tiles from {}", args.directory.display());

    let addr = ([127, 0, 0, 1], args.port).into();

    let make_service = make_service_fn(move |_conn| {
        let direcoty = args.directory.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                service::get_service(req, direcoty.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
