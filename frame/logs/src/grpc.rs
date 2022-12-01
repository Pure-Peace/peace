use peace_pb::frame::logs::{
    logs_rpc_server::LogsRpc, CommonRpcResult, SetLogLevelRequest,
    ToggleStatusRequest,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct LogsRpcService {}

#[tonic::async_trait]
impl LogsRpc for LogsRpcService {
    async fn set_log_level(
        &self,
        request: Request<SetLogLevelRequest>,
    ) -> Result<Response<CommonRpcResult>, Status> {
        let level = crate::level_from_int(request.into_inner().log_level)
            .map_err(|_| {
                Status::invalid_argument("Failed to convert level from int.")
            })?;
        crate::set_level(level)
            .map_err(|err| Status::internal(err.to_string()))?;

        info!("<LogsRpc> Reload log level to: [{}]", level);
        Ok(Response::new(CommonRpcResult { success: true, msg: None }))
    }

    async fn toggle_debug_mode(
        &self,
        request: Request<ToggleStatusRequest>,
    ) -> Result<Response<CommonRpcResult>, Status> {
        let enabled = request.into_inner().enabled;
        crate::toggle_debug_mode(enabled)
            .map_err(|err| Status::internal(err.to_string()))?;

        info!("<LogsRpc> Toggle debug mode: [{}]", enabled);
        Ok(Response::new(CommonRpcResult { success: true, msg: None }))
    }
}

#[cfg(test)]
mod test {
    use peace_pb::frame::logs::{
        logs_rpc_server::LogsRpc, LogLevel, SetLogLevelRequest,
        ToggleStatusRequest,
    };

    use crate::grpc::LogsRpcService;

    #[tokio::test]
    async fn try_set_level() {
        let s = LogsRpcService {};
        let req = tonic::Request::new(SetLogLevelRequest {
            log_level: LogLevel::Info as i32,
        });
        assert!(s.set_log_level(req).await.is_ok());

        let req = tonic::Request::new(SetLogLevelRequest { log_level: -1 });
        assert!(s.set_log_level(req).await.is_err());
    }

    #[tokio::test]
    async fn try_debug_mode() {
        let s = LogsRpcService {};
        let req = tonic::Request::new(ToggleStatusRequest { enabled: true });
        assert!(s.toggle_debug_mode(req).await.is_ok());

        let req = tonic::Request::new(ToggleStatusRequest { enabled: false });
        assert!(s.toggle_debug_mode(req).await.is_ok());
    }
}
