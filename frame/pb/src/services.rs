#[cfg(feature = "bancho")]
pub mod bancho {
    tonic::include_proto!("peace.services.bancho");

    #[cfg(feature = "reflection")]
    pub const BANCHO_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("bancho_descriptor");
}

#[cfg(feature = "peace_db")]
pub mod peace_db {
    tonic::include_proto!("peace.services.db.peace");

    #[cfg(feature = "reflection")]
    pub const PEACE_DB_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("peace_db_descriptor");
}
