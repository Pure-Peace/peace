use peace_db::peace::{entity::users::Model as User, Repository};
use peace_domain::peace::{Ascii, Unicode, Username};
use tonic::Status;

pub async fn get_user_model_by_username(
    repo: &Repository,
    username: Option<&str>,
    username_unicode: Option<&str>,
) -> Result<User, Status> {
    let username = username.and_then(|n| {
        Username::<Ascii>::from_str(n).ok().map(|n| n.safe_name())
    });
    let username_unicode = username_unicode.and_then(|n| {
        Username::<Unicode>::from_str(n).ok().map(|n| n.safe_name())
    });

    repo.find_user_by_username(username, username_unicode)
        .await
        .map_err(|err| Status::internal(err.to_string()))?
        .ok_or(Status::not_found("user not found"))
}
