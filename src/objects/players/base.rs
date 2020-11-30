use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerBase {
    pub id: i32,
    pub name: String,
    pub privileges: i32,
    pub country: String,
}
