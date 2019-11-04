use std::collections::HashMap;

use hyper::{Body, Request, Response, StatusCode};

use rusqlite::Connection;

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("404 page not found"))
        .unwrap()
}

fn bad_request(msg: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(msg))
        .unwrap()
}

pub fn get_service(
    connections: &'static HashMap<String, Connection>,
) -> impl Fn(Request<Body>) -> Response<Body> {
    move |request| {
        let path = request.uri().path();
        let response = match path {
            p if p.starts_with("/services") => {
                let segments: Vec<&str> = p.split('/').collect();
                if segments.len() == 2 {
                    // Root url: show all services
                    return not_found();
                }
                let tileset_name = segments[2];
                let connection = match connections.get(tileset_name) {
                    Some(c) => c,
                    None => return bad_request(format!("{} does not exist", tileset_name)),
                };
                Response::new(Body::from(format!("{}", p)))
            }
            _ => not_found(),
        };
        response
    }
}
