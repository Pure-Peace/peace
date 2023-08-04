use crate::ToLevelFilter;
use pb_base::{BoolValue, EmptyRequest, ExecSuccess, StringValue};
use pb_logs::{logs_rpc_server::LogsRpc, SetLevelRequest};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct LogsRpcService {}

#[tonic::async_trait]
impl LogsRpc for LogsRpcService {
    async fn set_level(
        &self,
        request: Request<SetLevelRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let level = request
            .into_inner()
            .level
            .to_level_filter()
            .map_err(Status::invalid_argument)?;

        crate::set_level(level)
            .map_err(|err| Status::internal(err.to_string()))?;

        info!("<LogsRpc> Set log level to: [{}]", level);
        Ok(Response::new(ExecSuccess::default()))
    }

    async fn set_debug_mode(
        &self,
        request: Request<BoolValue>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let enabled = request.into_inner().value;
        crate::toggle_debug_mode(enabled)
            .map_err(|err| Status::internal(err.to_string()))?;

        info!("<LogsRpc> Set debug mode: [{}]", enabled);
        Ok(Response::new(ExecSuccess::default()))
    }

    async fn set_env_filter(
        &self,
        request: Request<StringValue>,
    ) -> Result<Response<StringValue>, Status> {
        let filter = request.into_inner().value;
        crate::set_env_filter(&filter)
            .map_err(|err| Status::internal(err.to_string()))?;

        let current_filter = crate::env_filter(None).to_string();
        info!("<LogsRpc> Set env filter to: [{}]", current_filter);
        Ok(Response::new(StringValue { value: current_filter }))
    }

    async fn get_config(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<StringValue>, Status> {
        Ok(Response::new(StringValue {
            value: crate::env_filter(None).to_string(),
        }))
    }
}

#[cfg(test)]
mod test {
    use pb_base::BoolValue;
    use pb_logs::{logs_rpc_server::LogsRpc, LogLevel, SetLevelRequest};

    use crate::grpc::LogsRpcService;

    #[tokio::test]
    async fn try_set_level() {
        let svc = LogsRpcService {};
        let req = tonic::Request::new(SetLevelRequest {
            level: LogLevel::Info as i32,
        });
        assert!(svc.set_level(req).await.is_ok());

        let req = tonic::Request::new(SetLevelRequest { level: -1 });
        assert!(svc.set_level(req).await.is_err());
    }

    #[tokio::test]
    async fn try_debug_mode() {
        let svc = LogsRpcService {};
        let req = tonic::Request::new(BoolValue { value: true });
        assert!(svc.set_debug_mode(req).await.is_ok());

        let req = tonic::Request::new(BoolValue { value: false });
        assert!(svc.set_debug_mode(req).await.is_ok());
    }
}
