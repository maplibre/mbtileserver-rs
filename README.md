> Note: This project started as a partial implementation of the API and features provided by [mbtileserver](https://github.com/consbio/mbtileserver) written in Go by [Brendan Ward](https://github.com/brendan-ward). It might diverge from that project in the future.

# rust-mbtileserver

[![Crates.io](https://img.shields.io/crates/v/mbtileserver.svg)](https://crates.io/crates/mbtileserver)
[![Build Status](https://travis-ci.com/ka7eh/rust-mbtileserver.svg?branch=master)](https://travis-ci.com/ka7eh/rust-mbtileserver)
[![Coverage Status](https://coveralls.io/repos/github/ka7eh/rust-mbtileserver/badge.svg?branch=master)](https://coveralls.io/github/ka7eh/rust-mbtileserver?branch=master)

_Tested with rust 1.41_

A simple Rust-based server for map tiles stored in mbtiles format.

## Usage

Run `mbtileserver --help` for a list and description of the available flags:

```
mbtileserver 0.1.5
A simple mbtile server

USAGE:
    mbtileserver [FLAGS] [OPTIONS]

FLAGS:
        --disable-preview    Disable preview map
    -h, --help               Prints help information
    -V, --version            Prints version information

OPTIONS:
        --allowed-hosts <allowed_hosts>    A comma-separated list of allowed hosts [default: *]
    -d, --directory <directory>            Tiles directory [default: ./tiles]
    -H, --header <header>...               Add custom header
    -p, --port <port>                      Port [default: 3000]
```

Run `mbtileserver` to start serving the mbtiles in a given folder. The default folder is `./tiles` and you can change it with `-d` flag.
The server starts on port 3000 by default. You can use a different port via `-p` flag.

### Endpoints

| Endpoint                                                    | Description                                                                    |
|-------------------------------------------------------------|--------------------------------------------------------------------------------|
| /services                                                   | lists all discovered and valid mbtiles in the tiles directory                  |
| /services/<path-to-tileset>                                 | shows tileset metadata                                                         |
| /services/<path-to-tileset>/map                             | tileset preview                                                                |
| /services/<path-to-tileset>/tiles/{z}/{x}/{y}.<tile-format> | returns tileset tile at the given x, y, and z                                  |
| /services/<path-to-tileset>/tiles/{z}/{x}/{y}.json          | returns UTFGrid data at the given x, y, and z (only for tilesets with UTFGrid) |

## Docker

You can test this project by running `docker-compose up`. It starts a server on port 3000 and serves the tilesets in `./tiles` directory.
