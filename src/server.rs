use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use crate::service;
use crate::tiles::Tilesets;

#[tokio::main]
pub async fn run(
    port: u16,
    allowed_hosts: Vec<String>,
    headers: Vec<(String, String)>,
    disable_preview: bool,
    allow_reload_api: bool,
    allow_reload_signal: bool,
    tilesets: Tilesets,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([0, 0, 0, 0], port).into();
    let server = Server::try_bind(&addr)?;

    let service = {
        let tilesets = tilesets.clone();
        make_service_fn(move |_conn| {
            let tilesets = tilesets.clone();
            let allowed_hosts = allowed_hosts.clone();
            let headers = headers.clone();
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

    if allow_reload_signal {
        let tilesets = tilesets.clone();
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

    println!("Listening on http://{}", addr);
    server.serve(service).await?;

    Ok(())
}
