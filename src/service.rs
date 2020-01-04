use std::collections::HashMap;
use std::path::PathBuf;

use hyper::{header, Body, Request, Response, StatusCode};

use rusqlite::{params, Connection, OpenFlags, NO_PARAMS};

use serde_json;

use tera::{Context, Tera};

use crate::tiles;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("404 page not found"))
        .unwrap()
}

// fn bad_request(msg: String) -> Response<Body> {
//     Response::builder()
//         .status(StatusCode::BAD_REQUEST)
//         .body(Body::from(msg))
//         .unwrap()
// }

fn tiles_list(base_url: &str, tilesets: &HashMap<String, PathBuf>) -> Response<Body> {
    let mut resp: Vec<tiles::TileSummaryJSON> = Vec::new();
    for k in tilesets.keys() {
        resp.push(tiles::TileSummaryJSON {
            image_type: String::from("jpg"),
            url: format!("{}/{}", base_url, k),
        });
    }
    let resp_json = serde_json::to_string(&resp).unwrap(); // TODO handle error
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(resp_json))
        .unwrap() // TODO handle error
}

fn tile_details(base_url: &str, tile_name: &str, tile_path: &PathBuf) -> Response<Body> {
    let connection =
        Connection::open_with_flags(tile_path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();

    let tile_format = tiles::get_data_format(&connection);

    let mut statement = connection
        .prepare(r#"SELECT name, value FROM metadata"#)
        .unwrap();
    let mut metadata_rows = statement.query(NO_PARAMS).unwrap();

    let mut metadata = tiles::TileMetaJSON {
        name: None,
        version: None,
        map: format!("{}/{}/{}", base_url, tile_name, "map"),
        tiles: format!(
            "{}/{}/tiles/{{z}}/{{x}}/{{y}}.{}/{}",
            base_url, tile_name, tile_format, "query"
        ),
        tilejson: String::from("2.1.0"),
        scheme: String::from("xyz"),
        id: String::from(tile_name),
        format: tile_format,
        grids: None,
        bounds: None,
        minzoom: None,
        maxzoom: None,
        description: None,
        attribution: None,
        legend: None,
        template: None,
    };

    while let Some(row) = metadata_rows.next().unwrap() {
        let label: String = row.get(0).unwrap();
        let value: String = row.get(1).unwrap();
        match label.as_ref() {
            "name" => metadata.name = Some(value),
            "version" => metadata.version = Some(value),
            "bounds" => {
                metadata.bounds = Some(value.split(",").filter_map(|s| s.parse().ok()).collect())
            }
            "minzoom" => metadata.minzoom = Some(value.parse().unwrap()),
            "maxzoom" => metadata.maxzoom = Some(value.parse().unwrap()),
            "description" => metadata.description = Some(value),
            "attribution" => metadata.attribution = Some(value),
            "legend" => metadata.legend = Some(value),
            "template" => metadata.template = Some(value),
            _ => (),
        }
    }

    let resp_json = serde_json::to_string(&metadata).unwrap(); // TODO handle error
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(String::from(resp_json)))
        .unwrap() // TODO handle error
}

pub fn tile_map(base_url: &str, tile_name: &str) -> Response<Body> {
    let mut ctx = Context::new();
    ctx.insert("ID", tile_name);
    ctx.insert(
        "URL",
        &format!("{}/{}/tiles/{{z}}/{{y}}/{{x}}.png", base_url, tile_name),
    );
    let body = Body::from(TEMPLATES.render("map.html", &ctx).unwrap().to_string());
    Response::new(body)
}

pub fn tile_data(tile_path: &PathBuf, query: &[&str]) -> Response<Body> {
    let connection =
        Connection::open_with_flags(tile_path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let z = query[0];
    let y: u32 = (1 << z.parse::<u32>().unwrap()) - 1 - query[1].parse::<u32>().unwrap();
    let rest = query[2];
    let (x, format) = match rest.find(".") {
        Some(index) => (&rest[..index], &rest[index + 1..]),
        None => panic!(),
    };

    let mut statement = connection
        .prepare(
            r#"
                SELECT tile_data
                FROM map,
                     images
                WHERE zoom_level = ?1
                  AND tile_column = ?2
                  AND tile_row = ?3
                  AND map.tile_id = images.tile_id
                "#,
        )
        .unwrap(); // TODO handle error
    let tile_data: Vec<u8> = statement
        .query_row(params![z, x, y], |row| Ok(row.get(0).unwrap()))
        .unwrap_or(tiles::get_blank_image());

    Response::builder()
        .header(header::CONTENT_TYPE, tiles::get_content_type(format))
        .body(Body::from(tile_data))
        .unwrap() // TODO handle error
}

fn assets(path: &str) -> Response<Body> {
    let body = Body::from(TEMPLATES.render(path, &Context::new()).unwrap().to_string());
    Response::new(body)
}

pub async fn get_service(
    request: Request<Body>,
    directory: PathBuf,
) -> Result<Response<Body>, hyper::Error> {
    let tilesets = tiles::get_tiles(&directory);

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

    if path.starts_with("/services") {
        let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
        if segments.len() == 1 {
            // Root url: show all services
            return Ok(tiles_list(&base_url, &tilesets));
        }

        let tile_name = segments[1];
        let tile_path = tilesets.get(tile_name).unwrap(); // TODO handle error

        if segments[segments.len() - 1] == "map" {
            return Ok(tile_map(&base_url, tile_name));
        }

        if segments.len() == 2 {
            return Ok(tile_details(&base_url, tile_name, tile_path));
        }

        return Ok(tile_data(tile_path, &segments[3..]));
    }
    if path.starts_with("/static") {
        return Ok(assets(&path[1..]));
    }
    Ok(not_found())
}
