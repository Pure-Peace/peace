#[cfg(feature = "bancho")]
pub mod bancho {
    tonic::include_proto!("peace.services.bancho");
}

#[cfg(feature = "db")]
pub mod db {
    tonic::include_proto!("peace.services.db");
}
