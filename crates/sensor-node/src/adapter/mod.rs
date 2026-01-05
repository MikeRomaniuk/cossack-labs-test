pub mod grpc;

pub trait TelemetryAdapter {
    fn send_telemetry(&self, message: interface_types::Telemetry) -> impl Future<Output = anyhow::Result<()>> + Send;
}
