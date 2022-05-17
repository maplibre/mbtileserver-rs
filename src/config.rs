use std::collections::HashMap;
use std::path::PathBuf;

use clap::Parser;
use log::warn;

use crate::errors::{Error, Result};
use crate::tiles;

#[derive(Parser, Default, Debug)]
#[clap(about = "A simple mbtiles server")]
#[clap(version)]
pub struct Args {
    #[clap(long, short, default_value = "./tiles", help = "Tiles directory")]
    pub directory: PathBuf,
    #[clap(skip)]
    pub tilesets: HashMap<String, tiles::TileMeta>,
    #[clap(short, long, default_value_t = 3000, help = "Server port")]
    pub port: u16,
    #[clap(
        long,
        default_value = "localhost,127.0.0.1,[::1]",
        value_delimiter = ',',
        help = "\"*\" matches all domains and \".<domain>\" matches all subdomains for the given domain"
    )]
    pub allowed_hosts: Vec<String>,
    #[clap(
        short = 'H',
        long,
        help = "Add custom header. Can be used multiple times."
    )]
    pub header: Vec<String>,
    #[clap(skip)]
    pub headers: Vec<(String, String)>,
    #[clap(long, help = "Disable preview map")]
    pub disable_preview: bool,
}

impl Args {
    /// Update args after the initially parsing them with Clap
    pub fn post_parse(mut self) -> Result<Self> {
        if !self.directory.is_dir() {
            return Err(Error::Config(format!(
                "Directory does not exists: {}",
                self.directory.display()
            )));
        }
        self.tilesets = tiles::discover_tilesets(String::new(), &self.directory);
        self.allowed_hosts
            .iter_mut()
            .for_each(|v| *v = v.trim().to_string());

        for header in &self.header {
            let kv: Vec<&str> = header.split(':').collect();
            if kv.len() == 2 {
                let k = kv[0].trim();
                let v = kv[1].trim();
                if !k.is_empty() && !v.is_empty() {
                    self.headers.push((k.to_string(), v.to_string()))
                } else {
                    warn!("Invalid header: {header}");
                }
            } else {
                warn!("Invalid header: {header}");
            }
        }

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_missing_directory() {
        let dir = TempDir::new("tiles").unwrap();
        let dir_name = dir.path().to_str().unwrap().to_string();
        dir.close().unwrap();
        let args = Args::try_parse_from(&["", &format!("-d {dir_name}")])
            .unwrap()
            .post_parse();
        match args {
            Ok(_) => (),
            Err(err) => {
                assert!(format!("{err}").starts_with("Directory does not exists"));
            }
        };
    }

    #[test]
    fn test_valid_headers() {
        let args = Args::try_parse_from(&[
            "",
            "--header",
            "cache-control: public,max-age=14400",
            "--header",
            "access-control-allow-origin: *",
        ])
        .unwrap()
        .post_parse()
        .unwrap();
        println!("{:?}", args.headers);
        assert_eq!(
            args.headers,
            vec![
                (
                    "cache-control".to_string(),
                    "public,max-age=14400".to_string(),
                ),
                (
                    "access-control-allow-origin".to_string(),
                    "*".to_string(),
                )
            ]
        );
    }

    #[test]
    fn test_invalid_headers() {
        let app = Args::try_parse_from(&["", "-H"]);
        assert!(app.is_err());

        let args = Args::try_parse_from(&["", "-H k:"])
            .unwrap()
            .post_parse()
            .unwrap();
        assert_eq!(args.headers, vec![]);

        let args = Args::try_parse_from(&["", "-H :v"])
            .unwrap()
            .post_parse()
            .unwrap();
        assert_eq!(args.headers, vec![]);
    }
}
