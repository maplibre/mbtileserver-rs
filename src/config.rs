use std::env;
use std::path::PathBuf;
use std::time::Duration;

use clap::{crate_version, App, Arg, ArgMatches};
use regex::Regex;

use crate::errors::{Error, Result};
use crate::tiles;

lazy_static! {
    static ref DURATION_RE: Regex = Regex::new(r"\d+[smhd]").unwrap();
}

#[derive(Clone, Debug)]
pub struct Args {
    pub tilesets: tiles::Tilesets,
    pub port: u16,
    pub allowed_hosts: Vec<String>,
    pub headers: Vec<(String, String)>,
    pub disable_preview: bool,
    pub allow_reload_api: bool,
    pub allow_reload_signal: bool,
    pub reload_interval: Option<Duration>,
    pub disable_watcher: bool,
}

pub fn get_app<'a, 'b>() -> App<'a, 'b> {
    App::new("mbtileserver")
        .about("A simple mbtile server")
        .version(crate_version!())
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .default_value("./tiles")
                .help("Tiles directory\n")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .default_value("3000")
                .help("Server port\n")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("allowed_hosts")
                .long("allowed-hosts")
                .default_value("localhost, 127.0.0.1, [::1]")
                .help("A comma-separated list of allowed hosts")
                .long_help("\"*\" matches all domains and \".<domain>\" matches all subdomains for the given domain\n")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("header")
                .short("H")
                .long("header")
                .help("Add custom header\n")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("disable_preview")
                .long("disable-preview")
                .help("Disable preview map\n"),
        )
        .arg(
            Arg::with_name("allow_reload_api")
                .long("allow-reload-api")
                .help("Allow reloading tilesets with /reload endpoint\n"),
        )
        .arg(
            Arg::with_name("allow_reload_signal")
                .long("allow-reload-signal")
                .help("Allow reloading tilesets with a SIGHUP\n"),
        )
        .arg(
            Arg::with_name("reload_interval")
                .long("reload-interval")
                .help("An interval at which tilesets get reloaded")
                .long_help("\"*\" in 1h30m format\n")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("disable_watcher")
                .long("disable-watcher")
                .help("Disable fs watcher for automatic tileset reload\n"),
        )
}

pub fn parse(matches: ArgMatches) -> Result<Args> {
    let port = match matches.value_of("port").unwrap().parse::<u16>() {
        Ok(p) => p,
        Err(_) => {
            return Err(Error::Config(String::from(
                "Port must be a positive number",
            )))
        }
    };

    let tilesets = if let Some(directory_str) = matches.value_of("directory") {
        let directory = PathBuf::from(directory_str);
        if !directory.is_dir() {
            return Err(Error::Config(format!(
                "Directory does not exists: {}",
                directory_str
            )));
        }
        tiles::discover_tilesets(String::new(), directory)
    } else {
        return Err(Error::Config("Invalid value for directory".to_string()));
    };

    let allowed_hosts: Vec<String> = matches
        .value_of("allowed_hosts")
        .unwrap()
        .split(',')
        .map(|host| String::from(host.trim()))
        .collect();

    let mut headers = Vec::new();
    if let Some(headers_iter) = matches.values_of("header") {
        for header in headers_iter {
            let kv: Vec<&str> = header.split(':').collect();
            if kv.len() == 2 {
                let k = kv[0].trim();
                let v = kv[1].trim();
                if !k.is_empty() && !v.is_empty() {
                    headers.push((String::from(k), String::from(v)))
                } else {
                    warn!("Invalid header: {}", header);
                }
            } else {
                warn!("Invalid header: {}", header);
            }
        }
    }

    let reload_interval = match matches.value_of("reload_interval") {
        Some(str) => {
            let mut duration = Duration::ZERO;
            for mat in DURATION_RE.find_iter(str) {
                let mut mat = mat.as_str().to_owned();
                let char = mat.chars().nth(mat.len() - 1).unwrap();
                mat.truncate(mat.len() - 1);
                let multiplier = match char {
                    's' => 1,
                    'm' => 60,
                    'h' => 60 * 60,
                    'd' => 60 * 60 * 24,
                    _ => return Err(Error::Config("Invalid value for duration".to_string())),
                };
                let qty = match mat.parse::<u64>() {
                    Ok(v) => v,
                    Err(_) => return Err(Error::Config("Invalid value for duration".to_string())),
                };
                duration += Duration::from_secs(multiplier * qty);
            }
            Some(duration)
        }
        None => None,
    };

    let disable_preview = matches.occurrences_of("disable_preview") != 0;
    let allow_reload_api = matches.occurrences_of("allow_reload_api") != 0;
    let allow_reload_signal = matches.occurrences_of("allow_reload_signal") != 0;
    let disable_watcher = matches.occurrences_of("disable_watcher") != 0;

    Ok(Args {
        tilesets,
        port,
        allowed_hosts,
        headers,
        disable_preview,
        allow_reload_api,
        allow_reload_signal,
        reload_interval,
        disable_watcher,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_missing_directory() {
        let dir = TempDir::new("tiles").unwrap();
        let dir_name = String::from(dir.path().to_str().unwrap());
        dir.close().unwrap();
        match parse(get_app().get_matches_from(vec!["mbtileserver", &format!("-d {}", dir_name)])) {
            Ok(_) => (),
            Err(err) => {
                assert!(format!("{}", err).starts_with("Directory does not exists"));
            }
        };
    }

    #[test]
    fn test_valid_headers() {
        let args = parse(get_app().get_matches_from(vec![
            "mbtileserver",
            "--header=cache-control: public,max-age=14400",
            "--header=access-control-allow-origin: *",
        ]))
        .unwrap();
        assert_eq!(
            args.headers,
            vec![
                (
                    String::from("cache-control"),
                    String::from("public,max-age=14400")
                ),
                (
                    String::from("access-control-allow-origin"),
                    String::from("*")
                )
            ]
        );
    }

    #[test]
    fn test_invalid_headers() {
        let app = get_app().get_matches_from_safe(vec!["mbtileserver", "-H"]);
        assert!(app.is_err());

        let args = parse(get_app().get_matches_from(vec!["mbtileserver", "-H k:"])).unwrap();
        assert_eq!(args.headers, vec![]);

        let args = parse(get_app().get_matches_from(vec!["mbtileserver", "-H :v"])).unwrap();
        assert_eq!(args.headers, vec![]);
    }
}
