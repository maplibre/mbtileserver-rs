use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::collections::HashMap;

use crate::service;
use crate::tiles::TileMeta;

#[tokio::main]
pub async fn run(
    port: u16,
    allowed_hosts: Vec<String>,
    headers: Vec<(String, String)>,
    disable_preview: bool,
    tilesets: HashMap<String, TileMeta>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([0, 0, 0, 0], port).into();
    let server = Server::try_bind(&addr)?;

    let service = make_service_fn(move |_conn| {
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
                )
            }))
        }
    });

    println!("Listening on http://{addr}");
    server.serve(service).await?;

    Ok(())
}
