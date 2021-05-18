use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Clone, Debug, PostgresMapper)]
#[pg_mapper(table = "")]
pub struct PlayerAddress {
    pub id: i32,
    pub user_id: i32,
    pub adapters_hash: String,
    pub uninstall_id: String,
    pub disk_id: String,
    pub privileges: i32,
}
