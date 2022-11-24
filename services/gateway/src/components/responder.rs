use crate::components::{error::Error, router::AnyPathRouters};
use axum::{
    body::{Body, BoxBody},
    extract::Host,
    handler::Handler,
    http::Request,
    response::{IntoResponse, Response},
    Router,
};
use std::{convert::Infallible, marker::PhantomData};
use tower::{load_shed, timeout, BoxError, ServiceExt};

/// Route `/` handler.
pub async fn app_root() -> Response {
    tools::pkg_metadata!().into_response()
}

/// Route `/*path` handler.
pub async fn any_path(
    Host(hostname): Host,
    mut req: Request<Body>,
    any_routers: AnyPathRouters,
) -> Response {
    // Fix `axum 0.6.0-rc5` `src/extract/matched_path.rs:146` debug_assert panic.
    req.extensions_mut().remove::<axum::extract::MatchedPath>();

    let result = match hostname.as_str() {
        "c.peace.local" | "osu.peace.local" => {
            call_router(any_routers.bancho, req).await
        },
        _ => return Error::NotFound.into(),
    };

    result.into_response()
}

pub async fn handle_404() -> Response {
    Error::NotFound.into()
}

pub async fn call_router(
    router: Router,
    req: Request<Body>,
) -> Response<BoxBody> {
    router.into_service().oneshot(req).await.into_response()
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

/// The axum [`Handler`] can be wrapped into this structure.
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

#[cfg(test)]
mod test {
    use crate::bancho::impls::client;
    use crate::components::responder::HandlerWrapper;
    use std::collections::HashMap;

    #[test]
    fn wrap_handler() {
        let mut map = HashMap::<&str, HandlerWrapper>::new();

        map.insert("handler1", HandlerWrapper::new(client::ask_peppy));
        map.insert("handler2", HandlerWrapper::new(client::web::check_updates));
    }
}
