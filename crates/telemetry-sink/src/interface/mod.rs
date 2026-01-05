use interface_types::Telemetry;

pub(crate) trait TelemetryReceiver {
    fn recv(&mut self) -> impl Future<Output = Option<Telemetry>> + Send;
}

pub(crate) trait LogWriter<T> {
    fn log(&mut self, log: T) -> anyhow::Result<()>;
}
