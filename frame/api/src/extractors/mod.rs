use crate::error::Error;
use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, FromRequestParts},
    http::{request::Parts, Request},
    response::{IntoResponse, Response},
};
pub use axum_client_ip::InsecureClientIp as ClientIp;
use derive_deref::Deref;
use hyper::header::USER_AGENT;
use std::fmt::Display;

pub const OSU_VERSION: &str = "osu-version";
pub const OSU_TOKEN: &str = "osu-token";

/// Represents the version of the Bancho client.
#[derive(Debug, Deref, Serialize, Deserialize)]
pub struct BanchoClientVersion(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for BanchoClientVersion
where
    S: Send + Sync,
{
    type Rejection = Error;

    /// Extracts the `osu-version` header from the request parts and constructs
    /// a `BanchoClientVersion` instance from it.
    ///
    /// # Arguments
    ///
    /// * `parts` - A mutable reference to the `Parts` struct containing the
    ///             request parts.
    /// * `_state` - A reference to the application state.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `BanchoClientVersion` instance if
    /// successful, or an `Error` if the `osu-version` header is invalid.
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
            .ok_or(
                anyhow!("Invalid `osu-version` header, please check.").into(),
            )
    }
}

impl Display for BanchoClientVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Wrapper for the `osu-token` header value.
#[derive(Debug, Deref, Serialize, Deserialize)]
pub struct BanchoClientToken(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for BanchoClientToken
where
    S: Send + Sync,
{
    type Rejection = Error;

    /// Parses the `osu-token` header value from the incoming request and returns it
    /// as a `BanchoClientToken` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the `osu-token` header is not present or if it cannot be
    /// converted to a string.
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
            .ok_or(anyhow!("`osu-token` header is required.").into())
    }
}

impl Display for BanchoClientToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
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
    type Rejection = Response;

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
            .map(|ua| ua == "osu!")
            .unwrap_or(false)
        {
            return Err(Error::Anyhow(anyhow!("Invalid client user-agent."))
                .into_response());
        }

        // Extract the request body and wrap it in a `BanchoRequestBody`.
        Ok(Self(
            Bytes::from_request(req, state)
                .await
                .map_err(IntoResponse::into_response)?,
        ))
    }
}
