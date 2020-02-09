//! # rust-mbtileserver
//!
//! A simple Rust-based server for map tiles stored in mbtiles format.
//!
//! ## Usage
//!
//! Run `mbtileserver --help` for a list and description of the available flags:
//!
//! ```
//! MBTiles Server
//!
//! USAGE:
//!     mbtileserver [OPTIONS]
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!     -d, --directory <directory>    Tiles directory [default: ./tiles]
//!     -p, --port <port>              Port [default: 3000]
//! ```
//!
//! Run `mbtileserver` to start serving the mbtiles in a given folder. The default folder is `./tiles` and you can change it with `-d` flag.
//! The server starts on port 3000 by default. You can use a different port via `-p` flag.
//!
//! ### Endpoints
//!
//! | Endpoint                                                    | Description                                                                    |
//! |-------------------------------------------------------------|--------------------------------------------------------------------------------|
//! | /services                                                   | lists all discovered and valid mbtiles in the tiles directory                  |
//! | /services/<path-to-tileset>                                 | shows tileset metadata                                                         |
//! | /services/<path-to-tileset>/map                             | tileset preview                                                                |
//! | /services/<path-to-tileset>/tiles/{z}/{x}/{y}.<tile-format> | returns tileset tile at the given x, y, and z                                  |
//! | /services/<path-to-tileset>/tiles/{z}/{x}/{y}.json          | returns UTFGrid data at the given x, y, and z (only for tilesets with UTFGrid) |

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
