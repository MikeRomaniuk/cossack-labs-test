use grpc_types::service::telemetry as grpc;
use grpc_types::service::telemetry::telemetry_service_client::TelemetryServiceClient;
use interface_types::Telemetry;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;

use crate::adapter::TelemetryAdapter;

pub struct TelemetryClient {
    client: TelemetryServiceClient<tonic::transport::Channel>,
    telemetry_tx: Sender<grpc::SensorTelemetry>,
}

impl TelemetryClient {
    const STREAM_SIZE: usize = 10;

    pub async fn connect(dst: String) -> anyhow::Result<Self> {
        let client = TelemetryServiceClient::connect(dst).await?;
        let telemetry_tx = Self::open_telemetry_stream(client.clone());
        Ok(Self { client, telemetry_tx })
    }

    fn open_telemetry_stream(
        mut client: TelemetryServiceClient<tonic::transport::Channel>,
    ) -> mpsc::Sender<grpc::SensorTelemetry> {
        let (telemetry_tx, telemetry_rx) = mpsc::channel(Self::STREAM_SIZE);

        tokio::spawn(async move {
            client
                .open_telemetry_stream(ReceiverStream::new(telemetry_rx))
                .await
                .expect("Telemetry stream terminated by core")
        });

        telemetry_tx
    }
}

impl TelemetryAdapter for TelemetryClient {
    async fn send_telemetry(&self, message: Telemetry) -> anyhow::Result<()> {
        self.telemetry_tx.send(grpc::SensorTelemetry::from(message)).await?;

        Ok(())
    }
}
