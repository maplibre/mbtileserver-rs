use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::PathBuf;

use rusqlite::{Connection, OpenFlags};

pub fn get_connections(path: &PathBuf) -> HashMap<String, Connection> {
    let mut connections = HashMap::new();
    for file in read_dir(path).unwrap() {
        let file = file.unwrap().path();
        if file.extension().and_then(OsStr::to_str) == Some("mbtiles") {
            let file_name = file.file_stem().unwrap().to_str().unwrap();
            connections.insert(
                file_name.to_string(),
                Connection::open_with_flags(file, OpenFlags::SQLITE_OPEN_NO_MUTEX).unwrap(),
            );
        }
    }
    connections
}
