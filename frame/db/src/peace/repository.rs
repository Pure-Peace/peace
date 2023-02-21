use super::entity;

use entity::{users, users::Entity as User};
use sea_orm::*;

pub struct Repository;

impl Repository {
    /* pub async fn find_user_by_username() {
        User::find_by_id(values)
    } */

    pub async fn create_user(
        db: &DatabaseConnection,
        name: String,
        name_unicode: Option<String>,
        password: String,
        email: String,
        country: Option<String>,
    ) -> Result<InsertResult<users::ActiveModel>, DbErr> {
        fn to_safe(s: &String) -> String {
            s.trim().to_ascii_lowercase().replace(' ', "_")
        }

        User::insert(users::ActiveModel {
            name: Set(name.trim().to_owned()),
            name_safe: Set(to_safe(&name)),
            name_unicode: Set(name_unicode
                .as_ref()
                .map(|s| s.trim().to_owned())),
            name_unicode_safe: Set(name_unicode.as_ref().map(to_safe)),
            password: Set(password),
            email: Set(email),
            country: Set(country),
            ..Default::default()
        })
        .exec(db)
        .await
    }
}
