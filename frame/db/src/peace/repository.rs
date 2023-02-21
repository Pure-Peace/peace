use super::entity;

use entity::{users, users::Entity as User};
use peace_domain::peace::CreateUser;
use sea_orm::*;

pub struct Repository;

impl Repository {
    /* pub async fn find_user_by_username() {
        User::find_by_id(values)
    } */

    pub async fn create_user(
        db: &DatabaseConnection,
        creat_user: CreateUser,
    ) -> Result<InsertResult<users::ActiveModel>, DbErr> {
        User::insert(users::ActiveModel {
            name: Set(creat_user.name.as_ref().to_owned()),
            name_safe: Set(creat_user.name.safe_name().into()),
            name_unicode: Set(creat_user
                .name_unicode
                .as_ref()
                .map(|n| n.as_ref().to_owned())),
            name_unicode_safe: Set(creat_user
                .name_unicode
                .map(|u| u.safe_name().into())),
            password: Set(creat_user.password.into()),
            email: Set(creat_user.email.into()),
            country: Set(creat_user.country),
            ..Default::default()
        })
        .exec(db)
        .await
    }
}
