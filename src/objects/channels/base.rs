use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Clone, Debug, PostgresMapper)]
#[pg_mapper(table = "")]
pub struct ChannelBase {
    pub name: String,
    pub title: String,
    pub read_priv: i32,
    pub write_priv: i32,
    pub auto_join: bool,
}
