use tokio_util::sync::CancellationToken;

use crate::adapter::TelemetryAdapter;
use crate::infrastructure::config::Config;

pub struct SensorNode<S: TelemetryAdapter> {
    sender: S,
    config: Config,
}

impl<S: TelemetryAdapter> SensorNode<S> {
    pub fn new(sender: S, config: Config) -> Self {
        Self { sender, config }
    }

    fn compose_message(name: &str, value: u32) -> interface_types::Telemetry {
        interface_types::Telemetry {
            name: name.to_owned(),
            value,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub async fn run(&self, cancellation_token: CancellationToken) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1) / self.config.rate);
        let mut value = 0;

        interval.reset();

        tokio::select! {
            _ = interval.tick() => {
                let _ = self
                    .sender
                    .send_telemetry(Self::compose_message(&self.config.name, value))
                    .await
                    .inspect_err(|err| tracing::error!("Failed to send telemetry: {}", err));

                value = value.wrapping_add(1);
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("Cancellation token received, shutting down...");
            }
        }
    }
}
