use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelBase {
    pub name: String,
    pub title: String,
    pub read_priv: i32,
    pub write_priv: i32,
    pub auto_join: bool,
}
