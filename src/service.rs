use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use hyper::{header, Body, Request, Response, StatusCode};

use regex::Regex;

use rusqlite::{Connection, OpenFlags};

use serde_json;

use crate::tiles;
use crate::utils;

lazy_static! {
    static ref TILE_URL_RE: Regex =
        Regex::new(r"^/services/(?P<tile_path>.*)/tiles/(?P<z>\d+)/(?P<x>\d+)/(?P<y>\d+)\.(?P<format>[a-zA-Z]+)/?(\?(?P<query>.*))?").unwrap();
}

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOT_FOUND: &[u8] = b"Not Found";

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOT_FOUND.into())
        .unwrap()
}

fn server_error() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(INTERNAL_SERVER_ERROR.into())
        .unwrap()
}

fn bad_request(msg: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(msg))
        .unwrap()
}

pub fn tile_map(tile_path: &PathBuf) -> Response<Body> {
    let connection =
        Connection::open_with_flags(tile_path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let template = match tiles::get_data_format_via_query(&connection, "tile") {
        Ok(tile_format) => match tile_format {
            utils::DataFormat::PBF => "templates/map_vector.html",
            _ => "templates/map.html",
        },
        _ => return server_error(),
    };
    let file = File::open(template).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let body = Body::from(contents);
    Response::new(body)
}

fn assets(path: &str) -> Response<Body> {
    let file = match File::open(format!("templates/{}", path)) {
        Ok(file) => file,
        Err(_) => return not_found(),
    };
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let body = Body::from(contents);
    Response::new(body)
}

pub async fn get_service(
    request: Request<Body>,
    directory: PathBuf,
) -> Result<Response<Body>, hyper::Error> {
    let tilesets = tiles::discover_tilesets(String::from(""), &directory);

    let path = request.uri().path();
    let scheme = match request.uri().scheme_str() {
        Some(scheme) => format!("{}://", scheme),
        None => String::from("http://"),
    };
    let base_url = format!(
        "{}{}/services",
        scheme,
        request.headers()["host"].to_str().unwrap()
    );

    match TILE_URL_RE.captures(path) {
        Some(matches) => {
            let tile_path = tilesets
                .get(matches.name("tile_path").unwrap().as_str())
                .unwrap();
            let z = matches.name("z").unwrap().as_str().parse::<u32>().unwrap();
            let x = matches.name("x").unwrap().as_str().parse::<u32>().unwrap();
            let y = matches.name("y").unwrap().as_str().parse::<u32>().unwrap();
            let y: u32 = (1 << z) - 1 - y;
            let data_format = matches.name("format").unwrap().as_str();
            // For future use
            let _query_string = match matches.name("query") {
                Some(q) => q.as_str(),
                None => "",
            };

            return match data_format {
                "json" => match tiles::get_grid_data(tile_path, z, x, y) {
                    Ok(data) => {
                        let data = serde_json::to_vec(&data).unwrap();
                        Ok(Response::builder()
                            .header(header::CONTENT_TYPE, utils::DataFormat::JSON.content_type())
                            .header(header::CONTENT_ENCODING, "gzip")
                            .body(Body::from(utils::encode(&data)))
                            .unwrap())
                    }
                    Err(_) => Ok(not_found()),
                },
                _ => Ok(Response::builder()
                    .header(
                        header::CONTENT_TYPE,
                        utils::DataFormat::new(data_format).content_type(),
                    )
                    .body(Body::from(tiles::get_tile_data(tile_path, z, x, y)))
                    .unwrap()),
            };
        }
        None => {
            if path.starts_with("/services") {
                let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
                if segments.len() == 1 {
                    // Root url (/services): show all services
                    let resp_json =
                        serde_json::to_string(&tiles::get_tiles_list(&base_url, &tilesets))
                            .unwrap(); // TODO handle error
                    return Ok(Response::builder()
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(resp_json))
                        .unwrap()); // TODO handle error
                }

                if segments[segments.len() - 1] == "map" {
                    // Tileset map preview (/services/<tileset-path>/map)
                    let tile_name = segments[1..(segments.len() - 1)].join("/");
                    let tile_path = match tilesets.get(&tile_name) {
                        Some(tile_path) => tile_path,
                        None => {
                            return Ok(bad_request(format!(
                                "Tileset does not exist: {}",
                                tile_name
                            )))
                        }
                    };
                    return Ok(tile_map(tile_path));
                }

                // Tileset details (/services/<tileset-path>)
                let tile_name = segments[1..].join("/");
                let tile_path = match tilesets.get(&tile_name) {
                    Some(tile_path) => tile_path,
                    None => {
                        return Ok(bad_request(format!(
                            "Tileset does not exist: {}",
                            tile_name
                        )))
                    }
                };
                let query_string = match request.uri().query() {
                    Some(q) => format!("?{}", q),
                    None => String::new(),
                };

                match tiles::get_tile_details(&base_url, &tile_name, tile_path, query_string) {
                    Ok(metadata) => {
                        let resp_json = serde_json::to_string(&metadata).unwrap(); // TODO handle error
                        return Ok(Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(String::from(resp_json)))
                            .unwrap()); // TODO handle error
                    }
                    Err(_) => return Ok(server_error()),
                }
            }
            if path.starts_with("/static") {
                // Serve static files for map preview
                return Ok(assets(&path[1..]));
            }
        }
    };
    Ok(not_found())
}
