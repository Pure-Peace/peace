#[cfg(feature = "bancho")]
pub mod bancho {
    tonic::include_proto!("peace.services.bancho");

    pub const BANCHO_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("bancho_descriptor");
}

#[cfg(feature = "db")]
pub mod db {
    tonic::include_proto!("peace.services.db");

    pub const DB_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("db_descriptor");
}
