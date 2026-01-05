use core::error::Error as _;

use anyhow::Context as _;
use grpc_types::service::telemetry as grpc;
use grpc_types::service::telemetry::telemetry_service_client::TelemetryServiceClient;
use interface_types::Telemetry;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Certificate, ClientTlsConfig, Identity};

use crate::adapter::TelemetryAdapter;
use crate::infrastructure::config::TlsConfig;

pub(crate) struct TelemetryClient {
    _client: TelemetryServiceClient<tonic::transport::Channel>,
    telemetry_tx: Sender<grpc::SensorTelemetry>,
}

impl TelemetryClient {
    const STREAM_SIZE: usize = 10;

    pub(crate) async fn connect(dst: String, tls_config: Option<TlsConfig>) -> anyhow::Result<Self> {
        let client = if let Some(tls_config) = tls_config {
            tracing::info!("Configuring mTLS for client");

            let server_root_ca_cert = Certificate::from_pem(tls_config.ca);
            let client_identity = Identity::from_pem(tls_config.cert, tls_config.key);

            let tls = ClientTlsConfig::new()
                .domain_name("localhost")
                .ca_certificate(server_root_ca_cert)
                .identity(client_identity);

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
        Ok(Self {
            _client: client,
            telemetry_tx,
        })
    }

    fn open_telemetry_stream(
        mut client: TelemetryServiceClient<tonic::transport::Channel>,
    ) -> Sender<grpc::SensorTelemetry> {
        let (telemetry_tx, telemetry_rx) = mpsc::channel(Self::STREAM_SIZE);

        tokio::spawn(async move {
            match client.open_telemetry_stream(ReceiverStream::new(telemetry_rx)).await {
                Ok(response) => {
                    tracing::info!("Telemetry stream established successfully: {:?}", response);
                }
                Err(e) => {
                    tracing::error!("Telemetry stream error: {:?}", e);
                    tracing::error!("Error source: {:?}", e.source());
                }
            }
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
