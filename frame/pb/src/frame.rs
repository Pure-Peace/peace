#[cfg(feature = "logs")]
pub mod logs {
    tonic::include_proto!("peace.frame.logs");

    pub const LOGS_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("logs_descriptor");
}
