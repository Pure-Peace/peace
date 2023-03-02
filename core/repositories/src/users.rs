use peace_db::{peace::entity::users, *};
use peace_domain::users::{
    CreateUser, UsernameAscii, UsernameSafe, UsernameUnicode,
};
use std::sync::Arc;
use tonic::Status;

pub type DynUsersRepository = Arc<dyn UsersRepository + Send + Sync>;

#[async_trait]
pub trait UsersRepository {
    async fn get_user_model_by_username(
        &self,
        username: Option<&str>,
        username_unicode: Option<&str>,
    ) -> Result<users::Model, Status>;

    async fn find_user_by_username(
        &self,
        username: Option<UsernameSafe>,
        username_unicode: Option<UsernameSafe>,
    ) -> Result<Option<users::Model>, DbErr>;

    async fn create_user(
        &self,
        creat_user: CreateUser,
    ) -> Result<InsertResult<users::ActiveModel>, DbErr>;

    async fn change_user_password(
        &self,
        user_id: Option<i32>,
        username: Option<UsernameSafe>,
        username_unicode: Option<UsernameSafe>,
        password: String,
    ) -> Result<InsertResult<users::ActiveModel>, DbErr>;
}

#[derive(Debug, Default, Clone)]
pub struct UsersRepositoryImpl {
    conn: DatabaseConnection,
}

impl UsersRepositoryImpl {
    pub fn new(conn: DatabaseConnection) -> UsersRepositoryImpl {
        Self { conn }
    }

    pub fn into_service(self) -> DynUsersRepository {
        Arc::new(self) as DynUsersRepository
    }
}

#[async_trait]
impl UsersRepository for UsersRepositoryImpl {
    async fn get_user_model_by_username(
        &self,
        username: Option<&str>,
        username_unicode: Option<&str>,
    ) -> Result<users::Model, Status> {
        let username = username.and_then(|n| {
            UsernameAscii::from_str(n).ok().map(|n| n.safe_name())
        });
        let username_unicode = username_unicode.and_then(|n| {
            UsernameUnicode::from_str(n).ok().map(|n| n.safe_name())
        });

        self.find_user_by_username(username, username_unicode)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or(Status::not_found("user not found"))
    }

    async fn find_user_by_username(
        &self,
        username: Option<UsernameSafe>,
        username_unicode: Option<UsernameSafe>,
    ) -> Result<Option<users::Model>, DbErr> {
        users::Entity::find()
            .filter(
                Condition::any()
                    .add_option(
                        username.map(|name| {
                            users::Column::NameSafe.eq(name.as_ref())
                        }),
                    )
                    .add_option(username_unicode.map(|name_unicode| {
                        users::Column::NameUnicodeSafe.eq(name_unicode.as_ref())
                    })),
            )
            .one(&self.conn)
            .await
    }

    async fn create_user(
        &self,
        creat_user: CreateUser,
    ) -> Result<InsertResult<users::ActiveModel>, DbErr> {
        users::Entity::insert(users::ActiveModel {
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
        .exec(&self.conn)
        .await
    }

    async fn change_user_password(
        &self,
        user_id: Option<i32>,
        username: Option<UsernameSafe>,
        username_unicode: Option<UsernameSafe>,
        password: String,
    ) -> Result<InsertResult<users::ActiveModel>, DbErr> {
        let user = users::Entity::find()
            .filter(
                Condition::any()
                    .add_option(user_id.map(|id| users::Column::Id.eq(id)))
                    .add_option(
                        username.map(|name| {
                            users::Column::NameSafe.eq(name.as_ref())
                        }),
                    )
                    .add_option(username_unicode.map(|name_unicode| {
                        users::Column::NameUnicodeSafe.eq(name_unicode.as_ref())
                    })),
            )
            .one(&self.conn)
            .await?
            .ok_or(DbErr::Custom("user not found".into()))?;

        let mut model = user.into_active_model();

        model.password = ActiveValue::Set(password);

        model.update(&self.conn).await?;

        todo!()
    }
}

#[cfg(test)]
mod test {
    use peace_db::*;
    use peace_domain::users::UsernameAscii;

    use crate::users::{UsersRepository, UsersRepositoryImpl};

    #[tokio::test]
    async fn test_main() {
        peace_logs::fmt()
            .with_max_level(peace_logs::Level::DEBUG)
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
            UsersRepositoryImpl::new(db.clone())
                .find_user_by_username(
                    Some(UsernameAscii::from_str("test").unwrap().safe_name()),
                    None
                )
                .await
        );
    }
}
