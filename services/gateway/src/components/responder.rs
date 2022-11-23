use std::{convert::Infallible, marker::PhantomData};

use axum::{
    body::{Body, BoxBody},
    extract::Host,
    handler::Handler,
    http::Request,
    response::{IntoResponse, Response},
    Router,
};

use tower::{load_shed, timeout, BoxError, Service};

use crate::bancho;
use crate::components::error::Error;

pub async fn app_root() -> String {
    tools::pkg_metadata!()
}

pub async fn any_path(Host(hostname): Host, req: Request<Body>) -> Response {
    println!("hostname ----- {:?} \n\nreq -----{:?}", hostname, req);

    let result = match hostname.as_str() {
        "c.peace.local" | "osu.peace.local" => {
            call_router(bancho::routers::bancho_client_routes(), req).await
        },
        _ => return Error::NotFound.into(),
    };

    result.into_response()
}

pub async fn call_router(
    router: Router,
    req: Request<Body>,
) -> Result<Response<BoxBody>, ()> {
    router.into_service().call(req).await.map_err(|_| ())
}

pub async fn handler_404() -> Response {
    Error::NotFound.into()
}

pub async fn handle_error(error: BoxError) -> Error {
    if error.is::<timeout::error::Elapsed>() {
        return Error::Timeout;
    }

    if error.is::<load_shed::error::Overloaded>() {
        return Error::Unavailable;
    }

    anyhow::anyhow!("Unhandled internal error: {:?}", error).into()
}

pub struct HandlerWrapper<S = (), B = Body, E = Infallible> {
    boxed_handler: Box<dyn Send>,
    _a: PhantomData<B>,
    _b: PhantomData<E>,
    _c: PhantomData<S>,
}

impl HandlerWrapper {
    pub fn handler(&self) -> &Box<dyn Send> {
        &self.boxed_handler
    }
}

impl<S, B> HandlerWrapper<S, B, Infallible> {
    pub fn new<H, T>(handler: H) -> Self
    where
        H: Handler<T, S, B>,
    {
        Self {
            boxed_handler: Box::new(handler),
            _a: PhantomData,
            _b: PhantomData,
            _c: PhantomData,
        }
    }
}

#[test]
fn wrap_handler() {
    use crate::bancho::impls::client;
    use std::collections::HashMap;

    let mut map = HashMap::<&str, HandlerWrapper>::new();
    map.insert("handler1", HandlerWrapper::new(client::ask_peppy));
    map.insert("handler2", HandlerWrapper::new(client::web::check_updates));
}
