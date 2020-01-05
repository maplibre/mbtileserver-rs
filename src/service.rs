use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use hyper::{header, Body, Request, Response, StatusCode};

use regex::Regex;

use serde_json;

use crate::tiles;

lazy_static! {
    static ref TILE_URL_RE: Regex = Regex::new(r"^/services/(.*)/tiles/(\d+/\d+/\d+.*)").unwrap();
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

pub fn tile_map() -> Response<Body> {
    let file = File::open("templates/map.html").unwrap();
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
    let tilesets = tiles::get_tiles(String::from(""), &directory);

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
            let tile_path = tilesets.get(&matches[1]).unwrap();
            let query: Vec<&str> = matches[2].split('/').collect();
            let (tile_data, format) = tiles::tile_data(tile_path, &query);
            return Ok(Response::builder()
                .header(header::CONTENT_TYPE, tiles::get_content_type(&format))
                .body(Body::from(tile_data))
                .unwrap());
        }
        None => {
            if path.starts_with("/services") {
                let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
                if segments.len() == 1 {
                    // Root url (/services): show all services
                    let resp_json =
                        serde_json::to_string(&tiles::tiles_list(&base_url, &tilesets)).unwrap(); // TODO handle error
                    return Ok(Response::builder()
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(resp_json))
                        .unwrap()); // TODO handle error
                }

                if segments[segments.len() - 1] == "map" {
                    // Tileset map preview (/services/<tileset-path>/map)
                    return Ok(tile_map());
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
                match tiles::tile_details(&base_url, &tile_name, tile_path) {
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
