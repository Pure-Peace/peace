use std::convert::Infallible;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
pub use axum_client_ip::ClientIp;

pub const OSU_VERSION: &str = "osu-version";
pub const OSU_TOKEN: &str = "osu-token";

#[derive(Debug, Serialize, Deserialize)]
pub struct OsuVersion(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for OsuVersion
where
    S: Send + Sync,
{
    type Rejection = crate::error::Error;

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
pub struct OsuToken(pub Option<String>);

#[async_trait]
impl<S> FromRequestParts<S> for OsuToken
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(OSU_TOKEN)
            .and_then(|hv| hv.to_str().ok())
            .and_then(|s| Some(s.to_owned()));

        Ok(Self(token))
    }
}
