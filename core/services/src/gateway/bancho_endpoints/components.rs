use super::BanchoHttpError;
use peace_pb::bancho_state::CheckUserTokenRequest;
use std::str::FromStr;
use tonic::IntoRequest;
use tools::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanchoClientToken {
    pub user_id: i32,
    pub session_id: Ulid,
    pub signature: String,
}

impl BanchoClientToken {
    #[inline]
    pub fn content(&self) -> String {
        format!("{}.{}", self.user_id, self.session_id)
    }

    #[inline]
    pub fn encode_content(user_id: i32, session_id: &str) -> String {
        format!("{user_id}.{session_id}")
    }

    #[inline]
    pub fn encode(user_id: i32, session_id: &str, signature: &str) -> String {
        format!("{user_id}.{session_id}.{signature}")
    }
}

impl FromStr for BanchoClientToken {
    type Err = BanchoHttpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split('.').collect::<Vec<&str>>();
        if split.len() != 3 {
            return Err(BanchoHttpError::InvalidOsuTokenHeader);
        }

        let user_id = split[0]
            .parse::<i32>()
            .map_err(|_| BanchoHttpError::InvalidOsuTokenHeader)?;

        let session_id = Ulid::from_str(split[1])
            .map_err(|_| BanchoHttpError::InvalidOsuTokenHeader)?;

        let signature = split[2].to_string();

        Ok(Self { user_id, session_id, signature })
    }
}

impl IntoRequest<CheckUserTokenRequest> for BanchoClientToken {
    fn into_request(self) -> tonic::Request<CheckUserTokenRequest> {
        tonic::Request::new(CheckUserTokenRequest {
            user_id: self.user_id,
            session_id: self.session_id.to_string(),
            signature: self.signature,
        })
    }
}

impl std::fmt::Display for BanchoClientToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.user_id, self.session_id, self.signature)
    }
}
