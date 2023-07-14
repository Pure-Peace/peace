use crate::GetUserError;
use peace_db::{
    peace::{entity::users, Peace},
    *,
};
use peace_domain::users::{
    CreateUser, UsernameAscii, UsernameSafe, UsernameUnicode,
};
use std::sync::Arc;

pub type DynUsersRepository = Arc<dyn UsersRepository + Send + Sync>;

#[async_trait]
pub trait UsersRepository {
    async fn get_user(
        &self,
        user_id: Option<i32>,
        username: Option<&str>,
        username_unicode: Option<&str>,
    ) -> Result<users::Model, GetUserError>;

    async fn get_user_by_id(
        &self,
        user_id: i32,
    ) -> Result<users::Model, GetUserError>;

    async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<users::Model, GetUserError>;

    async fn get_user_by_username_unicode(
        &self,
        username_unicode: &str,
    ) -> Result<users::Model, GetUserError>;

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
    pub conn: DbConnection<Peace>,
}

impl UsersRepositoryImpl {
    pub fn new(conn: DbConnection<Peace>) -> UsersRepositoryImpl {
        Self { conn }
    }

    pub fn into_service(self) -> DynUsersRepository {
        Arc::new(self) as DynUsersRepository
    }
}

#[async_trait]
impl UsersRepository for UsersRepositoryImpl {
    async fn get_user(
        &self,
        user_id: Option<i32>,
        username: Option<&str>,
        username_unicode: Option<&str>,
    ) -> Result<users::Model, GetUserError> {
        users::Entity::find()
            .filter(
                Condition::any()
                    .add_option(
                        user_id.map(|user_id| users::Column::Id.eq(user_id)),
                    )
                    .add_option(username.map(|name| {
                        users::Column::NameSafe
                            .eq(UsernameAscii::to_safe_name(name))
                    }))
                    .add_option(username_unicode.map(|name_unicode| {
                        users::Column::NameUnicodeSafe
                            .eq(UsernameUnicode::to_safe_name(name_unicode))
                    })),
            )
            .one(self.conn.as_ref())
            .await
            .map_err(GetUserError::from)?
            .ok_or(GetUserError::UserNotExists)
    }

    async fn get_user_by_id(
        &self,
        user_id: i32,
    ) -> Result<users::Model, GetUserError> {
        users::Entity::find_by_id(user_id)
            .one(self.conn.as_ref())
            .await
            .map_err(GetUserError::from)?
            .ok_or(GetUserError::UserNotExists)
    }

    async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<users::Model, GetUserError> {
        users::Entity::find()
            .filter(
                Condition::any().add(
                    users::Column::NameSafe
                        .eq(UsernameAscii::to_safe_name(username)),
                ),
            )
            .one(self.conn.as_ref())
            .await
            .map_err(GetUserError::from)?
            .ok_or(GetUserError::UserNotExists)
    }

    async fn get_user_by_username_unicode(
        &self,
        username_unicode: &str,
    ) -> Result<users::Model, GetUserError> {
        users::Entity::find()
            .filter(
                Condition::any().add(
                    users::Column::NameUnicodeSafe
                        .eq(UsernameUnicode::to_safe_name(username_unicode)),
                ),
            )
            .one(self.conn.as_ref())
            .await
            .map_err(GetUserError::from)?
            .ok_or(GetUserError::UserNotExists)
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
        .exec(self.conn.as_ref())
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
            .one(self.conn.as_ref())
            .await?
            .ok_or(DbErr::Custom("user not found".into()))?;

        let mut model = user.into_active_model();

        model.password = ActiveValue::Set(password);

        model.update(self.conn.as_ref()).await?;

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
            UsersRepositoryImpl::new(DbConnection::from(db.clone()))
                .get_user(
                    None,
                    Some(&UsernameAscii::to_safe_name("test")),
                    None
                )
                .await
        );
    }
}
