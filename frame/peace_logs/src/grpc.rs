use crate::logs_rpc;

use tonic::{Request, Response, Status};

use logs_rpc::logs_rpc_server::LogsRpc;
use logs_rpc::{CommonRpcResult, DebugModeRequest, ReloadLevelRequest};

#[derive(Debug, Default)]
pub struct LogsRpcService {}

#[tonic::async_trait]
impl LogsRpc for LogsRpcService {
    async fn reload_level(
        &self,
        request: Request<ReloadLevelRequest>,
    ) -> Result<Response<CommonRpcResult>, Status> {
        let level = crate::level_from_int(request.into_inner().level).map_err(
            |_| Status::invalid_argument("Failed to convert level from int."),
        )?;
        crate::reload_level(level)
            .map_err(|err| Status::internal(err.to_string()))?;

        info!("<LogsRpc> Reload log level to: [{}]", level);
        Ok(Response::new(CommonRpcResult { status: true, msg: None }))
    }

    async fn debug_mode(
        &self,
        request: Request<DebugModeRequest>,
    ) -> Result<Response<CommonRpcResult>, Status> {
        let enabled = request.into_inner().enabled;
        crate::debug_mode(enabled)
            .map_err(|err| Status::internal(err.to_string()))?;

        info!("<LogsRpc> Toggle debug mode: [{}]", enabled);
        Ok(Response::new(CommonRpcResult { status: true, msg: None }))
    }
}

#[cfg(test)]
mod test {
    use crate::logs_rpc::{
        logs_rpc_server::LogsRpc, DebugModeRequest, Level, ReloadLevelRequest,
    };

    use crate::grpc::LogsRpcService;

    #[tokio::test]
    async fn try_reload_level() {
        let s = LogsRpcService {};
        let req = tonic::Request::new(ReloadLevelRequest {
            level: Level::Info as i32,
        });
        assert!(s.reload_level(req).await.is_ok());

        let req = tonic::Request::new(ReloadLevelRequest { level: -1 });
        assert!(s.reload_level(req).await.is_err());
    }

    #[tokio::test]
    async fn try_debug_mode() {
        let s = LogsRpcService {};
        let req = tonic::Request::new(DebugModeRequest { enabled: true });
        assert!(s.debug_mode(req).await.is_ok());

        let req = tonic::Request::new(DebugModeRequest { enabled: false });
        assert!(s.debug_mode(req).await.is_ok());
    }
}
