use crate::config::Args;
use crate::service;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

#[tokio::main]
pub async fn run(args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([0, 0, 0, 0], args.port).into();
    let server = Server::try_bind(&addr)?;

    let service = make_service_fn(move |_conn| {
        let tilesets = args.tilesets.clone();
        let allowed_hosts = args.allowed_hosts.clone();
        let headers = args.headers.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                service::get_service(
                    req,
                    tilesets.clone(),
                    allowed_hosts.clone(),
                    headers.clone(),
                    args.disable_preview,
                )
            }))
        }
    });

    println!("Listening on http://{addr}");
    server.serve(service).await?;

    Ok(())
}
