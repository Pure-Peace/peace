use super::{parser, BanchoHttpError};
use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, FromRequestParts},
    headers::HeaderName,
    http::{request::Parts, Request},
};
use derive_deref::Deref;
use hyper::header::USER_AGENT;
use peace_pb::bancho::LoginRequest;

pub static OSU_USER_AGENT: HeaderName = HeaderName::from_static("osu!");
pub static OSU_VERSION: HeaderName = HeaderName::from_static("osu-version");
pub static OSU_TOKEN: HeaderName = HeaderName::from_static("osu-token");

#[derive(Debug)]
pub struct OsuClientLoginBody(pub LoginRequest);

#[async_trait]
impl<S, B> FromRequest<S, B> for OsuClientLoginBody
where
    Bytes: FromRequest<S, B>,
    LoginRequest: FromRequest<S, B>,
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = BanchoHttpError;

    async fn from_request(
        req: Request<B>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let body = BanchoRequestBody::from_request(req, state).await?.0;

        Ok(Self(
            parser::parse_osu_login_request_body(body.into())
                .map_err(|err| BanchoHttpError::LoginFailed(err.into()))?,
        ))
    }
}

/// Represents the version of the Bancho client.
#[derive(Debug, Deref, Serialize, Deserialize)]
pub struct BanchoClientVersion(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for BanchoClientVersion
where
    S: Send + Sync,
{
    type Rejection = BanchoHttpError;

    /// Extracts the `osu-version` header from the request parts and constructs
    /// a `BanchoClientVersion` instance from it.
    ///
    /// Returns a `Result` containing the `BanchoClientVersion` instance if
    /// successful, or an `Error` if the `osu-version` header is invalid.
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(&OSU_VERSION)
            .and_then(|hv| hv.to_str().ok())
            .map(|s| s.to_owned())
            .map(Self)
            .ok_or(BanchoHttpError::InvalidOsuVersionHeader)
    }
}

impl std::fmt::Display for BanchoClientVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Wrapper for the `osu-token` header value.
#[derive(Debug, Deref, Serialize, Deserialize)]
pub struct OsuTokenHeader(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for OsuTokenHeader
where
    S: Send + Sync,
{
    type Rejection = BanchoHttpError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(&OSU_TOKEN)
            .and_then(|hv| hv.to_str().ok())
            .map(|s| OsuTokenHeader(s.to_owned()))
            .ok_or(BanchoHttpError::InvalidOsuTokenHeader)
    }
}

/// A wrapper around the body of a Bancho request.
#[derive(Debug, Deref)]
pub struct BanchoRequestBody(pub Bytes);

#[async_trait]
impl<S, B> FromRequest<S, B> for BanchoRequestBody
where
    Bytes: FromRequest<S, B>,
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = BanchoHttpError;

    /// Attempts to extract the request body from the provided `Request`.
    async fn from_request(
        req: Request<B>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Ensure the request came from a valid client.
        if !req
            .headers()
            .get(USER_AGENT)
            .and_then(|hv| hv.to_str().ok())
            .map(|ua| ua == OSU_USER_AGENT.as_str())
            .unwrap_or(false)
        {
            return Err(BanchoHttpError::InvalidUserAgentHeader);
        }

        // Extract the request body and wrap it in a `BanchoRequestBody`.
        Ok(Self(
            Bytes::from_request(req, state)
                .await
                .map_err(|_| BanchoHttpError::ParseRequestError)?,
        ))
    }
}
