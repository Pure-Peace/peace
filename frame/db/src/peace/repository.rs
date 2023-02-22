use super::entity;
use entity::{users, users::Entity as User};
use peace_dal::*;
use peace_domain::peace::{Ascii, CreateUser, Unicode, Username, UsernameSafe};

#[derive(Debug, Clone)]
pub struct Repository(DatabaseConnection);

impl Repository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self(db)
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.0
    }

    pub async fn find_user_by_username(
        &self,
        username: Option<UsernameSafe>,
        username_unicode: Option<UsernameSafe>,
    ) -> Result<Option<users::Model>, DbErr> {
        User::find()
            .apply_if(username, |query, name| {
                query.filter(users::Column::NameSafe.eq(name.as_ref()))
            })
            .apply_if(username_unicode, |query, name_unicode| {
                query.filter(
                    users::Column::NameUnicodeSafe.eq(name_unicode.as_ref()),
                )
            })
            .one(self.conn())
            .await
    }

    pub async fn create_user(
        &self,
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
        .exec(self.conn())
        .await
    }
}

#[cfg(test)]
mod test {
    use crate::peace::Repository;
    use peace_dal::*;
    use peace_domain::peace::{Ascii, Username};

    #[tokio::test]
    async fn test_main() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .init();

        let db = Database::connect(ConnectOptions::from(
            "postgresql://postgres:123456@localhost:5432/peace",
        ))
        .await
        .unwrap();

        test1(&db).await;
    }

    async fn test1(db: &DatabaseConnection) {
        println!(
            "{:?}",
            Repository::new(db.clone())
                .find_user_by_username(
                    Some(
                        Username::<Ascii>::from_str("test")
                            .unwrap()
                            .safe_name()
                    ),
                    None
                )
                .await
        );
    }
}
