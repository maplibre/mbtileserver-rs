use std::path::PathBuf;

extern crate clap;
use clap::{App, Arg};

pub struct Args {
    pub directory: PathBuf,
    pub port: u16,
}

pub fn parse<'a>() -> Args {
    let matches = App::new("MBTiles Server")
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .help("Tiles directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Port")
                .takes_value(true),
        )
        .get_matches();

    let port = match matches.value_of("port") {
        Some(p) => p.parse::<u16>().unwrap_or(3000),
        None => 3000,
    };

    Args {
        directory: PathBuf::from(matches.value_of("directory").unwrap_or("tiles")),
        port,
    }
}
