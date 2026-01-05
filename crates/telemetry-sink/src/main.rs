use anyhow::Context as _;
use clap::Parser as _;
use tonic::transport::{Certificate, Identity, ServerTlsConfig};

use crate::infrastructure::cli::Args;
use crate::infrastructure::config::Config;

mod adapter;
mod domain;
mod infrastructure;
mod interface;

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();

    setup_logging().context("unable to initialize logging")?;

    let rt = if let Some(worker_threads) = cli.worker_threads {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(worker_threads)
            .enable_all()
            .build()
    } else {
        tokio::runtime::Builder::new_multi_thread().enable_all().build()
    }
    .expect("Failed to create tokio runtime");

    let config = Config::try_from(cli)?;

    rt.block_on(async move {
        let _ = tokio_main(config)
            .await
            .inspect_err(|err| tracing::error!("Sensor Node failed with an error: {}", err));
    });

    Ok(())
}

fn setup_logging() -> anyhow::Result<()> {
    use tracing::metadata::LevelFilter;
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::prelude::*;

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("TELEMETRY_SINK_LOG")
        .from_env_lossy();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false);
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .context("failed to set tracing global subscriber")?;

    Ok(())
}

async fn tokio_main(config: Config) -> anyhow::Result<()> {
    use grpc_types::service::telemetry::telemetry_service_server::TelemetryServiceServer;
    use tokio_util::sync::CancellationToken;
    use tonic::transport::Server;

    use crate::adapter::{FileLogger, TelemetryServer, TelemetryService};
    use crate::domain::TelemetryService as DomainTelemetryService;

    tracing::info!("Starting telemetry sink");

    let cancellation_token = CancellationToken::new();

    let (tx, rx) = tokio::sync::mpsc::channel(config.buffer_size);

    let grpc_server = TelemetryServer::new(tx);

    let telemetry_receiver = TelemetryService::new(rx);

    let file_logger = FileLogger::new(&config.log_file).context("Failed to create file logger")?;

    let mut domain_service = DomainTelemetryService::new(telemetry_receiver, file_logger, config.clone());

    let cancellation_token_clone = cancellation_token.clone();
    let domain_handle = tokio::spawn(async move {
        domain_service.run(cancellation_token_clone).await;
    });

    let addr = config.ip;
    tracing::info!("Starting gRPC server on {}", addr);

    let server_cancellation_token = cancellation_token.clone();
    let tls_config_clone = config.tls_config.clone();
    let server_handle = tokio::spawn(async move {
        async fn start_server(
            addr: core::net::SocketAddr,
            grpc_server: TelemetryServer,
            tls_config: Option<infrastructure::config::TlsConfig>,
            cancellation_token: CancellationToken,
        ) -> anyhow::Result<()> {
            let mut server_builder = if let Some(tls_config) = tls_config {
                let cert = tls_config.cert;
                let key = tls_config.key;
                let server_identity = Identity::from_pem(cert, key);

                let client_ca_cert = tls_config.ca;
                let client_ca_cert = Certificate::from_pem(client_ca_cert);

                let tls = ServerTlsConfig::new()
                    .identity(server_identity)
                    .client_ca_root(client_ca_cert);
                Server::builder().tls_config(tls).context("Failed to configure TLS")?
            } else {
                tracing::info!("No TLS configuration provided, starting without mTLS");
                Server::builder()
            };

            server_builder
                .add_service(TelemetryServiceServer::new(grpc_server))
                .serve_with_shutdown(addr, cancellation_token.cancelled())
                .await
                .context("gRPC server failed")
        }

        start_server(addr, grpc_server, tls_config_clone, server_cancellation_token).await
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal");
            cancellation_token.cancel();
        }
        result = server_handle => {
            match result {
                Ok(Ok(())) => tracing::info!("gRPC server stopped"),
                Ok(Err(e)) => tracing::error!("gRPC server error: {}", e),
                Err(e) => tracing::error!("Server task panicked: {}", e),
            }
        }
    }

    // Wait for domain service to finish
    let _ = domain_handle.await;

    tracing::info!("Telemetry sink stopped");

    Ok(())
}
