use anyhow::Context;
use grpc_types::service::telemetry as grpc;
use grpc_types::service::telemetry::telemetry_service_client::TelemetryServiceClient;
use interface_types::Telemetry;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;

use crate::adapter::TelemetryAdapter;
use crate::infrastructure::config::TlsConfig;

pub struct TelemetryClient {
    client: TelemetryServiceClient<tonic::transport::Channel>,
    telemetry_tx: Sender<grpc::SensorTelemetry>,
}

impl TelemetryClient {
    const STREAM_SIZE: usize = 10;

    pub async fn connect(dst: String, tls_config: Option<TlsConfig>) -> anyhow::Result<Self> {
        let client = if let Some(tls_config) = tls_config {
            tracing::info!("Configuring mTLS for client");

            let cert = tokio::fs::read(&tls_config.cert).await
                .context("Failed to read client certificate")?;
            let key = tokio::fs::read(&tls_config.key).await
                .context("Failed to read client private key")?;
            let ca = tokio::fs::read(&tls_config.ca).await
                .context("Failed to read CA certificate")?;

            let client_identity = tonic::transport::Identity::from_pem(cert, key);
            let ca_cert = tonic::transport::Certificate::from_pem(ca);

            let tls = tonic::transport::ClientTlsConfig::new()
                .identity(client_identity)
                .ca_certificate(ca_cert);

            let channel = tonic::transport::Channel::from_shared(dst)
                .context("Failed to parse server address")?
                .tls_config(tls)
                .context("Failed to configure TLS")?
                .connect()
                .await
                .context("Failed to connect to server")?;

            TelemetryServiceClient::new(channel)
        } else {
            TelemetryServiceClient::connect(dst).await?
        };

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
