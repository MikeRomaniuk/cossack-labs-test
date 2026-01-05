use grpc_types::service::telemetry::SensorTelemetry;
use interface_types::Telemetry;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, Streaming};

use crate::interface::TelemetryReceiver;

pub(crate) struct TelemetryService {
    channel: mpsc::Receiver<Telemetry>,
}

impl TelemetryService {
    pub(crate) fn new(channel: mpsc::Receiver<Telemetry>) -> Self {
        Self { channel }
    }
}

impl TelemetryReceiver for TelemetryService {
    async fn recv(&mut self) -> Option<Telemetry> {
        self.channel.recv().await
    }
}

pub(crate) struct TelemetryServer {
    channel: mpsc::Sender<Telemetry>,
}

impl TelemetryServer {
    pub(crate) fn new(channel: mpsc::Sender<Telemetry>) -> Self {
        Self { channel }
    }
}

#[tonic::async_trait]
impl grpc_types::service::telemetry::telemetry_service_server::TelemetryService for TelemetryServer {
    async fn open_telemetry_stream(
        &self,
        request: Request<Streaming<SensorTelemetry>>,
    ) -> Result<Response<grpc_types::service::telemetry::Result>, Status> {
        tracing::info!("=== TELEMETRY STREAM REQUEST RECEIVED ===");
        tracing::info!("Request metadata: {:?}", request.metadata());

        let reply = grpc_types::service::telemetry::Result {
            result: grpc_types::service::telemetry::ResultType::Ok.into(),
        };

        let mut messages = request.into_inner();
        loop {
            let message = messages.message().await;

            match message {
                Ok(Some(message)) => {
                    tracing::info!(data = ?message, "Received node sensor update");

                    match Telemetry::try_from(message) {
                        Ok(telemetry) => {
                            let _ = self
                                .channel
                                .send(telemetry)
                                .await
                                .inspect_err(|_| tracing::warn!("Failed to send telemetry message to domain"));
                        }
                        Err(e) => tracing::warn!("Failed to convert telemetry message: {e}"),
                    }
                }
                Ok(None) | Err(_) => {
                    // channel closed -> driver exited
                    tracing::warn!("Sensor node exited!");
                    break;
                }
            }
        }

        Ok(Response::new(reply))
    }
}
