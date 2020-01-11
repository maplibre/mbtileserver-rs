use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::PathBuf;

use rusqlite::{params, Connection, OpenFlags, NO_PARAMS};

use serde;
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileSummaryJSON {
    pub image_type: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileMetaJSON {
    pub name: Option<String>,
    pub version: Option<String>,
    pub map: String,
    pub tiles: Vec<String>,
    pub tilejson: String,
    pub scheme: String,
    pub id: String,
    pub format: String,
    pub grids: Option<Vec<String>>,
    pub bounds: Option<Vec<f64>>,
    pub minzoom: Option<u32>,
    pub maxzoom: Option<u32>,
    pub description: Option<String>,
    pub attribution: Option<String>,
    pub legend: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UTFGrid {
    key_name: String,
    key_json: String,
}

pub fn discover_tilesets(parent_dir: String, path: &PathBuf) -> HashMap<String, PathBuf> {
    // Walk through the given path and its subfolders, find all valid mbtiles and create and return a map of mbtiles file names to their absolute path
    let mut tiles = HashMap::new();
    for p in read_dir(path).unwrap() {
        let p = p.unwrap().path();
        if p.is_dir() {
            let dir_name = p.file_stem().unwrap().to_str().unwrap();
            let mut parent_dir_cloned = parent_dir.clone();
            parent_dir_cloned.push_str(dir_name);
            parent_dir_cloned.push_str("/");
            tiles.extend(discover_tilesets(parent_dir_cloned, &p));
        } else if p.extension().and_then(OsStr::to_str) == Some("mbtiles") {
            let file_name = p.file_stem().unwrap().to_str().unwrap();
            match Connection::open_with_flags(&p, OpenFlags::SQLITE_OPEN_READ_ONLY) {
                Ok(connection) => match get_data_format(&connection, "tile") {
                    Ok(_) => {
                        let mut parent_dir_cloned = parent_dir.clone();
                        parent_dir_cloned.push_str(file_name);
                        tiles.insert(parent_dir_cloned, p)
                    }
                    _ => None,
                },
                _ => None,
            };
        }
    }
    tiles
}

pub fn tiles_list(base_url: &str, tilesets: &HashMap<String, PathBuf>) -> Vec<TileSummaryJSON> {
    let mut tile_summary_json: Vec<TileSummaryJSON> = Vec::new();
    for (k, v) in tilesets.iter() {
        match Connection::open_with_flags(v, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            Ok(connection) => match get_data_format(&connection, "tile") {
                Ok(image_type) => tile_summary_json.push(TileSummaryJSON {
                    image_type,
                    url: format!("{}/{}", base_url, k),
                }),
                _ => (),
            },
            _ => (),
        }
    }
    tile_summary_json
}

pub fn tile_details(base_url: &str, tile_name: &str, tile_path: &PathBuf) -> Result<TileMetaJSON> {
    let connection =
        Connection::open_with_flags(tile_path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();

    let tile_format = match get_data_format(&connection, "tile") {
        Ok(tile_format) => tile_format,
        _ => return Err(Error),
    };

    let mut statement = connection
        .prepare(r#"SELECT name, value FROM metadata"#)
        .unwrap();
    let mut metadata_rows = statement.query(NO_PARAMS).unwrap();

    let grids = match get_grid_info(&connection) {
        Some(_) => Some(vec![format!(
            "{}/{}/tiles/{{z}}/{{x}}/{{y}}.json",
            base_url, tile_name
        )]),
        None => None,
    };

    let mut metadata = TileMetaJSON {
        name: None,
        version: None,
        map: format!("{}/{}/{}", base_url, tile_name, "map"),
        tiles: vec![format!(
            "{}/{}/tiles/{{z}}/{{x}}/{{y}}.{}",
            base_url, tile_name, tile_format
        )],
        tilejson: String::from("2.1.0"),
        scheme: String::from("xyz"),
        id: String::from(tile_name),
        format: tile_format,
        grids,
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

    Ok(metadata)
}

fn get_grid_info(connection: &Connection) -> Option<String> {
    let mut statement = connection.prepare(r#"SELECT count(*) FROM sqlite_master WHERE name IN ('grids', 'grid_data', 'grid_utfgrid', 'keymap', 'grid_key')"#).unwrap();
    let count: u8 = statement
        .query_row(NO_PARAMS, |row| {
            let value: u8 = row.get(0).unwrap();
            Ok(value)
        })
        .unwrap();
    if count == 5 {
        match get_data_format(connection, "grid") {
            Ok(grid_format) => return Some(grid_format),
            Err(_) => return None,
        };
    }
    None
}

pub fn grid_data(tile_path: &PathBuf, z: &str, x: &str, y: &str, query: &str) {
    let connection =
        Connection::open_with_flags(tile_path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let y: u32 = (1 << z.parse::<u32>().unwrap()) - 1 - y.parse::<u32>().unwrap();

    let mut statement = connection
        .prepare(
            r#"SELECT key_name, key_json
                FROM grid_data
                WHERE zoom_level = ?1
                  AND tile_column = ?2
                  AND tile_row = ?3
            "#,
        )
        .unwrap(); // TODO handle error
    let grid_data_iter = statement
        .query_map(params![z, x, y], |row| {
            Ok(UTFGrid {
                key_name: row.get(0).unwrap(),
                key_json: row.get(1).unwrap(),
            })
        })
        .unwrap();
    for gd in grid_data_iter {
        println!("Got data {:?}", gd.unwrap());
    }
}

pub fn tile_data(tile_path: &PathBuf, z: &str, x: &str, y: &str, query: &str) -> Vec<u8> {
    let connection =
        Connection::open_with_flags(tile_path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let y: u32 = (1 << z.parse::<u32>().unwrap()) - 1 - y.parse::<u32>().unwrap();

    let mut statement = connection
        .prepare(
            r#"SELECT tile_data
                FROM map, images
                WHERE zoom_level = ?1
                  AND tile_column = ?2
                  AND tile_row = ?3
                  AND map.tile_id = images.tile_id
            "#,
        )
        .unwrap(); // TODO handle error
    statement
        .query_row(params![z, x, y], |row| Ok(row.get(0).unwrap()))
        .unwrap_or(get_blank_image())
}

pub fn get_data_format(connection: &Connection, category: &str) -> Result<String> {
    let query = match category {
        "tile" => r#"SELECT tile_data FROM tiles LIMIT 1"#,
        "grid" => r#"SELECT grid_utfgrid FROM grid_utfgrid LIMIT 1"#,
        _ => return Err(Error),
    };
    let mut statement = match connection.prepare(query) {
        Ok(s) => s,
        Err(_) => return Err(Error),
    };
    let data_format: &str = statement
        .query_row(NO_PARAMS, |row| {
            let value: Vec<u8> = row.get(0).unwrap();
            match value.as_slice() {
                v if &v[0..2] == b"\x1f\x8b" => Ok("GZIP"), // this masks PBF format too
                v if &v[0..2] == b"\x78\x9c" => Ok("ZLIB"),
                v if &v[0..8] == b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A" => Ok("PNG"),
                v if &v[0..3] == b"\xFF\xD8\xFF" => Ok("JPG"),
                v if &v[0..14] == b"\x52\x49\x46\x46\xc0\x00\x00\x00\x57\x45\x42\x50\x56\x50" => {
                    Ok("WEBP")
                }
                _ => Ok("Unknown"),
            }
        })
        .unwrap();
    match data_format {
        f if f == "Unknown" => Err(Error),
        f => Ok(String::from(f).to_lowercase()),
    }
}

pub fn get_content_type(tile_format: &str) -> String {
    match tile_format {
        "png" => String::from("image/png"),
        "jpg" | "jpeg" => String::from("image/jpg"),
        "pbf" => String::from("application/x-protobuf"),
        "webp" => String::from("image/webp"),
        _ => String::from(""),
    }
}

pub fn get_blank_image() -> Vec<u8> {
    let image = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x01\x00\x00\x00\x01\x00\x01\x03\x00\x00\x00f\xbc:%\x00\x00\x00\x03PLTE\x00\x00\x00\xa7z=\xda\x00\x00\x00\x01tRNS\x00@\xe6\xd8f\x00\x00\x00\x1fIDATh\xde\xed\xc1\x01\r\x00\x00\x00\xc2 \xfb\xa76\xc77`\x00\x00\x00\x00\x00\x00\x00\x00q\x07!\x00\x00\x01\xa7W)\xd7\x00\x00\x00\x00IEND\xaeB`\x82";
    image.to_vec()
}
