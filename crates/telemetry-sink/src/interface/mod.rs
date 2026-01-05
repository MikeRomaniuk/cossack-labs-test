use std::future::Future;

use interface_types::Telemetry;

pub trait TelemetryReceiver {
    fn recv(&mut self) -> impl Future<Output = Option<Telemetry>> + Send;
}

pub trait LogWriter<T> {
    fn log(&mut self, log: T) -> anyhow::Result<()>;
}
