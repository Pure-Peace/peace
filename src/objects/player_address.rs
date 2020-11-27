use serde::Deserialize;
#[derive(Clone, Debug, Deserialize)]
pub struct PlayerAddress {
    pub id: i32,
    pub user_id: i32,
    pub apadaters_hash: String,
    pub uninstall_id: String,
    pub disk_id: String,
}
