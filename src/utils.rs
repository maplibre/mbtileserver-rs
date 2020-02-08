use std::io::prelude::*;

use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::GzEncoder;
use flate2::Compression;

use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataFormat {
    PNG,
    JPG,
    WEBP,
    JSON,
    PBF,
    GZIP,
    ZLIB,
    UNKNOWN,
}

impl DataFormat {
    pub fn new(format: &str) -> DataFormat {
        match format {
            "png" => DataFormat::PNG,
            "jpg" | "jpeg" => DataFormat::JPG,
            "webp" => DataFormat::WEBP,
            "json" => DataFormat::JSON,
            "pbf" => DataFormat::PBF,
            "gzip" => DataFormat::GZIP,
            "zlib" => DataFormat::ZLIB,
            _ => DataFormat::UNKNOWN,
        }
    }

    pub fn format(&self) -> &str {
        match *self {
            DataFormat::PNG => "png",
            DataFormat::JPG => "jpg",
            DataFormat::WEBP => "webp",
            DataFormat::JSON => "json",
            DataFormat::PBF => "pbf",
            DataFormat::GZIP => "",
            DataFormat::ZLIB => "",
            DataFormat::UNKNOWN => "",
        }
    }

    pub fn content_type(&self) -> &str {
        match *self {
            DataFormat::PNG => "image/png",
            DataFormat::JPG => "image/jpeg",
            DataFormat::WEBP => "image/webp",
            DataFormat::JSON => "application/json",
            DataFormat::PBF => "application/x-protobuf",
            DataFormat::GZIP => "",
            DataFormat::ZLIB => "",
            DataFormat::UNKNOWN => "",
        }
    }
}

pub fn decode(data: Vec<u8>, data_type: DataFormat) -> Result<String> {
    match data_type {
        DataFormat::GZIP => {
            let mut z = GzDecoder::new(&data[..]);
            let mut s = String::new();
            z.read_to_string(&mut s).unwrap();
            Ok(s)
        }
        DataFormat::ZLIB => {
            let mut z = ZlibDecoder::new(&data[..]);
            let mut s = String::new();
            z.read_to_string(&mut s).unwrap();
            Ok(s)
        }
        _ => Err(Error::InvalidDataFormat(String::from(data_type.format()))),
    }
}

pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(data).unwrap();
    e.finish().unwrap()
}

pub fn get_data_format(data: &Vec<u8>) -> DataFormat {
    match data {
        v if &v[0..2] == b"\x1f\x8b" => DataFormat::GZIP,
        v if &v[0..2] == b"\x78\x9c" => DataFormat::ZLIB,
        v if &v[0..8] == b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A" => DataFormat::PNG,
        v if &v[0..3] == b"\xFF\xD8\xFF" => DataFormat::JPG,
        v if &v[0..14] == b"\x52\x49\x46\x46\xc0\x00\x00\x00\x57\x45\x42\x50\x56\x50" => {
            DataFormat::WEBP
        }
        _ => DataFormat::UNKNOWN,
    }
}

pub fn get_blank_image() -> Vec<u8> {
    let image = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x01\x00\x00\x00\x01\x00\x01\x03\x00\x00\x00f\xbc:%\x00\x00\x00\x03PLTE\x00\x00\x00\xa7z=\xda\x00\x00\x00\x01tRNS\x00@\xe6\xd8f\x00\x00\x00\x1fIDATh\xde\xed\xc1\x01\r\x00\x00\x00\xc2 \xfb\xa76\xc77`\x00\x00\x00\x00\x00\x00\x00\x00q\x07!\x00\x00\x01\xa7W)\xd7\x00\x00\x00\x00IEND\xaeB`\x82";
    image.to_vec()
}
