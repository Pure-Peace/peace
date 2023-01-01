use crate::error::Error;
use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, FromRequestParts},
    http::{request::Parts, Request},
    response::{IntoResponse, Response},
};
pub use axum_client_ip::ClientIp;
use hyper::header::USER_AGENT;

pub const OSU_VERSION: &str = "osu-version";
pub const OSU_TOKEN: &str = "osu-token";

#[derive(Debug, Serialize, Deserialize)]
pub struct OsuVersion(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for OsuVersion
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(OSU_VERSION)
            .and_then(|hv| hv.to_str().ok())
            .and_then(|s| Some(s.to_owned()))
            .map(Self)
            .ok_or(anyhow!("Invalid osu-version header, please check.").into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OsuToken(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for OsuToken
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(OSU_TOKEN)
            .and_then(|hv| hv.to_str().ok())
            .and_then(|s| Some(s.to_owned()))
            .map(Self)
            .ok_or(anyhow!("osu-token header is required.").into())
    }
}

#[derive(Debug)]
pub struct OsuClientBody(pub Bytes);

#[async_trait]
impl<S, B> FromRequest<S, B> for OsuClientBody
where
    Bytes: FromRequest<S, B>,
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(
        req: Request<B>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        if !req
            .headers()
            .get(USER_AGENT)
            .and_then(|hv| hv.to_str().ok())
            .map(|ua| ua == "osu!")
            .unwrap_or(false)
        {
            return Err(Error::Anyhow(anyhow!("Invalid client user-agent."))
                .into_response());
        }

        Ok(Self(
            Bytes::from_request(req, state)
                .await
                .map_err(IntoResponse::into_response)?,
        ))
    }
}
