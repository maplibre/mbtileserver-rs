I started `rust-mbtileserver` in order to learn and practice rust. It's still a work in progress and is not stable.

This project tries to (partially) implement the API and features provided by [mbtileserver](https://github.com/consbio/mbtileserver) written in Go by [Brendan Ward](https://github.com/brendan-ward). If you are looking for a well-maintained, stable, and reliable mbtiles server with more features, check out the other repo.

If you'd like to help with this project, either to learn rust or improve the codebase, feel free to open tickets and submit pull requests.

## Development

After cloning the repo, run `cargo run` to start a server on port 3000. You can access the list of tilesets at `/services`. The default tile folder is `./tiles`, which can be overwritten with `-d` flag.

## Docker

You can test this project by running `docker-compose up`. It uses the tilesets in `./tiles`.
