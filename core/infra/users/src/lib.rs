use tools::Ulid;

pub mod sessions;
pub mod users_store;

pub use sessions::*;
pub use users_store::*;

pub trait UserKey {
    fn session_id(&self) -> Ulid;
    fn user_id(&self) -> i32;
    fn username(&self) -> String;
    fn username_unicode(&self) -> Option<String>;
}
