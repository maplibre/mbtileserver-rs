use log::error;

mod config;
mod errors;
mod server;
mod service;
mod tiles;
mod utils;

fn main() {
    pretty_env_logger::init_timed();

    let args = match config::parse(config::get_app().get_matches()) {
        Ok(args) => args,
        Err(err) => {
            error!("{}", err);
            std::process::exit(1)
        }
    };

    if let Err(e) = server::run(
        args.port,
        args.allowed_hosts,
        args.headers,
        args.disable_preview,
        args.tilesets,
    ) {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
