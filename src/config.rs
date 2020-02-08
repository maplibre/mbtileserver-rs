use std::fs::read_dir;
use std::path::PathBuf;

use clap::{crate_version, App, Arg};

use crate::errors::{Error, Result};

#[derive(Clone)]
pub struct Args {
    pub directory: PathBuf,
    pub port: u16,
}

pub fn parse() -> Result<Args> {
    let matches = App::new("mbtileserver")
        .about("A simple mbtile server")
        .version(crate_version!())
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .default_value("./tiles")
                .help("Tiles directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .default_value("3000")
                .help("Port")
                .takes_value(true),
        )
        .get_matches();

    let port = match matches.value_of("port").unwrap().parse::<u16>() {
        Ok(p) => p,
        Err(_) => {
            return Err(Error::Config(String::from(
                "Port must be a positive number",
            )))
        }
    };

    let directory = PathBuf::from(matches.value_of("directory").unwrap());
    match read_dir(directory.clone()) {
        Ok(_) => (),
        Err(_) => return Err(Error::Config(String::from("Directory does not exists"))),
    };

    Ok(Args { directory, port })
}
