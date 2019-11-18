use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::PathBuf;

use rusqlite::{Connection, NO_PARAMS};

pub fn get_tiles(path: &PathBuf) -> HashMap<String, PathBuf> {
    let mut tiles = HashMap::new();
    for file in read_dir(path).unwrap() {
        let file = file.unwrap().path();
        if file.extension().and_then(OsStr::to_str) == Some("mbtiles") {
            let file_name = file.file_stem().unwrap().to_str().unwrap();
            tiles.insert(file_name.to_string(), file);
        }
    }
    tiles
}

pub fn get_tile_format(connection: &Connection) -> String {
    let mut statement = connection
        .prepare(r#"SELECT tile_data FROM tiles LIMIT 1"#)
        .unwrap();
    let tile_format: &str = statement
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
    String::from(tile_format).to_lowercase()
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
