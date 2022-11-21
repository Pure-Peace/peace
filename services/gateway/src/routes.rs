use axum::{response::IntoResponse, http::Request, body::Body};

pub async fn root() -> String {
    tools::pkg_metadata!()
}


pub async fn bancho(req: Request<Body>) -> impl IntoResponse {
    println!("{:?}", req)
}
