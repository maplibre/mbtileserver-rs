use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;

use r2d2::Error as R2D2Error;
use rusqlite::Error as DBError;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    DBConnection(DBError),
    Pool(R2D2Error),
    Config(String),
    MissingTable(String),
    InvalidDataFormat(String),
    InvalidDataFormatQueryCategory(String),
    UnknownTileFormat(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Config(message) => write!(f, "{message}"),
            Error::MissingTable(tile_name) => {
                write!(f, "Missing tiles or metadata tables: {tile_name}")
            }
            Error::InvalidDataFormat(data_format) => {
                write!(f, "Invalid data format: {data_format}")
            }
            Error::InvalidDataFormatQueryCategory(tile_name) => {
                write!(f, "Invalid query category: {tile_name}")
            }
            Error::UnknownTileFormat(tile_name) => write!(f, "Unknown tile format: {tile_name}"),
            Error::DBConnection(_) => write!(f, "Database connection error"),
            Error::Pool(_) => write!(f, "Database pool connection error"),
        }
    }
}

impl StdError for Error {}
