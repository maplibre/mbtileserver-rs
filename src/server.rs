use crate::config::Args;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use crate::service;

#[tokio::main]
pub async fn run(args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([0, 0, 0, 0], args.port).into();
    let server = Server::try_bind(&addr)?;

    let service = {
        let args = args.clone();
        make_service_fn(move |_conn| {
            let tilesets = args.tilesets.clone();
            let allowed_hosts = args.allowed_hosts.clone();
            let headers = args.headers.clone();
            let disable_preview = args.disable_preview;
            let allow_reload_api = args.allow_reload_api;
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    service::get_service(
                        req,
                        tilesets.clone(),
                        allowed_hosts.clone(),
                        headers.clone(),
                        disable_preview,
                        allow_reload_api,
                    )
                }))
            }
        })
    };

    if args.allow_reload_signal {
        let tilesets = args.tilesets.clone();
        println!("Reloading on SIGHUP");
        tokio::spawn(async move {
            let mut handler =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()).unwrap();
            loop {
                handler.recv().await;
                println!("Caught SIGHUP, reloading tilesets");
                tilesets.reload();
            }
        });
    }

    if let Some(interval) = args.reload_interval {
        let tilesets = args.tilesets.clone();
        println!("Reloading every {} seconds", interval.as_secs());
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);
            loop {
                timer.tick().await;
                tilesets.reload();
            }
        });
    }

    println!("Listening on http://{}", addr);
    server.serve(service).await?;

    Ok(())
}
