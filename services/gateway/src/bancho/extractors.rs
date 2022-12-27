use super::parser;
use axum::{
    async_trait, body::Bytes, extract::FromRequest, http::Request,
    response::Response,
};
use peace_api::extractors::OsuClientBody;
use peace_pb::services::bancho::LoginRequest;

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
    type Rejection = Response;

    async fn from_request(
        req: Request<B>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let body = OsuClientBody::from_request(req, state).await?.0;

        let lines = parser::parse_osu_login_data_lines(body.to_vec())?;

        Ok(Self(parser::parse_osu_login_request_data(lines)?))
    }
}
