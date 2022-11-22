use axum::{body::Body, extract::Host, http::Request, response::IntoResponse};

pub async fn root() -> String {
    tools::pkg_metadata!()
}

pub async fn bancho(req: Request<Body>) -> impl IntoResponse {
    println!("{:?}", req)
}

pub async fn any_path(
    Host(hostname): Host,
    request: Request<Body>,
) -> impl IntoResponse {
    println!("{:?} \n\n{:?}", hostname, request);
    match hostname.as_str() {
        _ => todo!(),
    }
}
