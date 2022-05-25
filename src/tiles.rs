use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use log::warn;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use tilejson::{tilejson, Bounds, Center, TileJSON};

use crate::errors::{Error, Result};

use crate::utils::{decode, get_data_format, DataFormat};

type Connection = r2d2::PooledConnection<SqliteConnectionManager>;

#[derive(Clone, Debug)]
pub struct TileMeta {
    pub connection_pool: r2d2::Pool<SqliteConnectionManager>,
    pub path: PathBuf,
    pub tilejson: TileJSON,
    pub id: String,
    pub tile_format: DataFormat,
    pub grid_format: Option<DataFormat>,
    pub layer_type: Option<String>,
    pub json: Option<JSONValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileSummaryJSON {
    pub image_type: DataFormat,
    pub url: String,
}

#[derive(Deserialize)]
struct UTFGridKeys {
    pub grid: Vec<String>,
    pub keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UTFGrid {
    pub data: HashMap<String, JSONValue>,
    pub grid: Vec<String>,
    pub keys: Vec<String>,
}

pub fn get_data_format_via_query(
    tile_name: &str,
    connection: &Connection,
    category: &str,
) -> Result<DataFormat> {
    let query = match category {
        "tile" => r#"SELECT tile_data FROM tiles LIMIT 1"#,
        "grid" => r#"SELECT grid_utfgrid FROM grid_utfgrid LIMIT 1"#,
        _ => return Err(Error::InvalidDataFormatQueryCategory(tile_name.to_string())),
    };
    let mut statement = match connection.prepare(query) {
        Ok(s) => s,
        Err(err) => return Err(Error::DBConnection(err)),
    };
    let data_format: DataFormat = statement
        .query_row([], |row| {
            Ok(get_data_format(&row.get::<_, Vec<u8>>(0).unwrap()))
        })
        .unwrap_or(DataFormat::Unknown);
    Ok(data_format)
}

pub fn get_tile_details(path: &Path, tile_name: &str) -> Result<TileMeta> {
    let manager = SqliteConnectionManager::file(path).with_flags(OpenFlags::SQLITE_OPEN_READ_ONLY);
    let connection_pool = match r2d2::Pool::new(manager) {
        Ok(connection_pool) => connection_pool,
        Err(err) => return Err(Error::Pool(err)),
    };

    let connection = connection_pool.get().unwrap();

    // 'tiles', 'metadata' tables or views must be present
    let query = r#"SELECT count(*) FROM sqlite_master WHERE name IN ('tiles', 'metadata')"#;
    let mut statement = match connection.prepare(query) {
        Ok(s) => s,
        Err(err) => return Err(Error::DBConnection(err)),
    };
    match statement.query_row([], |row| Ok(row.get::<_, i8>(0).unwrap_or(0))) {
        Ok(count) => {
            if count < 2 {
                return Err(Error::MissingTable(tile_name.to_string()));
            }
        }
        Err(err) => return Err(Error::DBConnection(err)),
    };

    let tile_format = match get_data_format_via_query(tile_name, &connection, "tile") {
        Ok(tile_format) => match tile_format {
            DataFormat::Unknown => return Err(Error::UnknownTileFormat(tile_name.to_string())),
            DataFormat::Gzip => DataFormat::Pbf, // GZIP masks PBF format too
            _ => tile_format,
        },
        Err(err) => return Err(err),
    };

    let mut metadata = TileMeta {
        connection_pool,
        path: PathBuf::from(path),
        tilejson: tilejson! {
            tilejson: "2.1.0".to_string(),
            tiles: vec!["".to_string()],
        },
        id: tile_name.to_string(),
        tile_format,
        grid_format: get_grid_info(tile_name, &connection),
        layer_type: None,
        json: None,
    };

    let mut statement = connection
        .prepare(r#"SELECT name, value FROM metadata WHERE value IS NOT ''"#)
        .unwrap();
    let mut metadata_rows = statement.query([]).unwrap();

    while let Some(row) = metadata_rows.next().unwrap() {
        let label: String = row.get(0).unwrap();
        let value: String = row.get(1).unwrap();
        match label.as_ref() {
            "name" => metadata.tilejson.name = Some(value),
            "version" => metadata.tilejson.version = Some(value),
            "bounds" => metadata.tilejson.bounds = Some(Bounds::from_str(value.as_str()).unwrap()),
            "center" => metadata.tilejson.center = Some(Center::from_str(value.as_str()).unwrap()),
            "minzoom" => metadata.tilejson.minzoom = Some(value.parse().unwrap()),
            "maxzoom" => metadata.tilejson.maxzoom = Some(value.parse().unwrap()),
            "description" => metadata.tilejson.description = Some(value),
            "attribution" => metadata.tilejson.attribution = Some(value),
            "type" => metadata.layer_type = Some(value),
            "legend" => metadata.tilejson.legend = Some(value),
            "template" => metadata.tilejson.template = Some(value),
            "json" => metadata.json = Some(serde_json::from_str(&value).unwrap()),
            _ => (),
        };
    }

    Ok(metadata)
}

/// Walk through the given path and its subfolders, find all valid mbtiles and create
/// and return a map of mbtiles file names to their absolute path
pub fn discover_tilesets(parent_dir: String, path: &PathBuf) -> HashMap<String, TileMeta> {
    let mut tiles = HashMap::new();
    for p in read_dir(path).unwrap() {
        let p = p.unwrap().path();
        if p.is_dir() {
            let dir_name = p.file_stem().unwrap().to_str().unwrap();
            let mut parent_dir_cloned = parent_dir.clone();
            parent_dir_cloned.push_str(dir_name);
            parent_dir_cloned.push('/');
            tiles.extend(discover_tilesets(parent_dir_cloned, &p));
        } else if p.extension().and_then(OsStr::to_str) == Some("mbtiles") {
            let file_name = p.file_stem().and_then(OsStr::to_str).unwrap();
            let mut parent_dir_cloned = parent_dir.clone();
            parent_dir_cloned.push_str(file_name);
            match get_tile_details(&p, file_name) {
                Ok(tile_meta) => tiles.insert(parent_dir_cloned, tile_meta),
                Err(err) => {
                    warn!("{err}");
                    None
                }
            };
        }
    }
    tiles
}

fn get_grid_info(tile_name: &str, connection: &Connection) -> Option<DataFormat> {
    let mut statement = connection.prepare(r#"SELECT count(*) FROM sqlite_master WHERE name IN ('grids', 'grid_data', 'grid_utfgrid', 'keymap', 'grid_key')"#).unwrap();
    let count: u8 = statement
        .query_row([], |row| Ok(row.get(0).unwrap()))
        .unwrap();
    if count == 5 {
        return match get_data_format_via_query(tile_name, connection, "grid") {
            Ok(grid_format) => Some(grid_format),
            Err(err) => {
                warn!("{err}");
                None
            }
        };
    }
    None
}

pub fn get_grid_data(
    connection: &Connection,
    data_format: DataFormat,
    z: u32,
    x: u32,
    y: u32,
) -> Result<UTFGrid> {
    let mut statement = connection
        .prepare(
            r#"SELECT grid
                 FROM grids
                WHERE zoom_level = ?1
                  AND tile_column = ?2
                  AND tile_row = ?3
            "#,
        )
        .unwrap();
    let grid_data = match statement.query_row(params![z, x, y], |row| {
        Ok(row.get::<_, Vec<u8>>(0).unwrap())
    }) {
        Ok(d) => d,
        Err(err) => return Err(Error::DBConnection(err)),
    };
    let grid_key_json: UTFGridKeys =
        serde_json::from_str(&decode(grid_data, data_format).unwrap()).unwrap();
    let mut grid_data = UTFGrid {
        data: HashMap::new(),
        grid: grid_key_json.grid,
        keys: grid_key_json.keys,
    };

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
            Ok((
                row.get::<_, String>(0).unwrap(),
                row.get::<_, String>(1).unwrap(),
            ))
        })
        .unwrap();
    for gd in grid_data_iter {
        let (key, value) = gd.unwrap();
        let value: JSONValue = serde_json::from_str(&value).unwrap();
        grid_data.data.insert(key, value);
    }

    Ok(grid_data)
}

pub fn get_tile_data(connection: &Connection, z: u32, x: u32, y: u32) -> Result<Vec<u8>> {
    let mut statement = connection
        .prepare(
            r#"SELECT tile_data
                 FROM tiles
                WHERE zoom_level = ?1
                  AND tile_column = ?2
                  AND tile_row = ?3
            "#,
        )
        .unwrap(); // TODO handle error
    match statement.query_row(params![z, x, y], |row| Ok(row.get(0).unwrap())) {
        Ok(data) => Ok(data),
        Err(err) => Err(Error::DBConnection(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_list_of_valid_tilesets() {
        let tilesets = discover_tilesets(String::new(), &PathBuf::from("./tiles"));
        // 2 out of 7 tilesets in ./tiles directory are invalid
        assert_eq!(tilesets.len(), 5);

        assert!(!tilesets.contains_key("invalid"));
        assert!(!tilesets.contains_key("invalid-tile-format"));
    }

    #[test]
    fn get_tileset_metadata() {
        let tileset_details = get_tile_details(
            &PathBuf::from("./tiles/geography-class-png.mbtiles"),
            "geography-class-png",
        )
        .unwrap();
        // The rhs values are from metadata table of geography-class-png.mbtiles
        assert_eq!(tileset_details.tilejson.name.unwrap(), "Geography Class");
        assert_eq!(tileset_details.tilejson.version.unwrap(), "1.0.0");
        assert_eq!(tileset_details.tilejson.minzoom.unwrap(), 0);
        assert_eq!(tileset_details.tilejson.maxzoom.unwrap(), 1);
        assert_eq!(
            tileset_details.tilejson.bounds.unwrap(),
            Bounds::new(-180.0, -85.0511, 180.0, 85.0511)
        );
        assert_eq!(
            tileset_details.tilejson.center.unwrap(),
            Center::new(0.0, 20.0, 0)
        );
        assert_eq!(tileset_details.tile_format, DataFormat::Png);

        let tileset_details = get_tile_details(
            &PathBuf::from("./tiles/world_cities.mbtiles"),
            "world_cities",
        )
        .unwrap();
        // The rhs values are from metadata table of world_cities.mbtiles
        assert_eq!(
            tileset_details.tilejson.name.unwrap(),
            "Major cities from Natural Earth data"
        );
        assert_eq!(tileset_details.tilejson.version.unwrap(), "2");
        assert_eq!(tileset_details.tilejson.minzoom.unwrap(), 0);
        assert_eq!(tileset_details.tilejson.maxzoom.unwrap(), 6);
        assert_eq!(
            tileset_details.tilejson.bounds.unwrap(),
            Bounds::new(-123.123590, -37.818085, 174.763027, 59.352706)
        );
        assert_eq!(
            tileset_details.tilejson.center.unwrap(),
            Center::new(-75.937500, 38.788894, 6)
        );
        assert_eq!(tileset_details.tile_format, DataFormat::Pbf);
    }
}
