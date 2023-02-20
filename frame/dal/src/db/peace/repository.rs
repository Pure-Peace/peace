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
        name_safe: String,
        name_unicode: Option<String>,
        name_unicode_safe: Option<String>,
        password: String,
        email: String,
        country: Option<String>,
    ) -> Result<InsertResult<users::ActiveModel>, DbErr> {
        User::insert(users::ActiveModel {
            name: Set(name),
            name_safe: Set(name_safe),
            name_unicode: Set(name_unicode),
            name_unicode_safe: Set(name_unicode_safe),
            password: Set(password),
            email: Set(email),
            country: Set(country),
            ..Default::default()
        })
        .exec(db)
        .await
    }
}
