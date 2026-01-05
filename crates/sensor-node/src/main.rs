use anyhow::Context as _;
use clap::Parser as _;
use infrastructure::cli::Args;
use infrastructure::config::Config;

mod adapter;
mod domain;
mod infrastructure;

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

    let config = Config::try_from(cli.clone())?;

    rt.block_on(async move {
        let _ = tokio_main(config, cli.telemetry_sink_address)
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
        .with_env_var("SENSOR_NODE_LOG")
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

async fn tokio_main(node_config: Config, sink_address: String) -> anyhow::Result<()> {
    tracing::info!("Starting sensor node");
    tracing::info!("Sink address: {}", sink_address);

    let adapter = adapter::grpc::TelemetryClient::connect(sink_address, node_config.tls_config.clone()).await?;
    let sensor_node = domain::SensorNode::new(adapter, node_config);

    let cancellation_token = tokio_util::sync::CancellationToken::new();
    let child = cancellation_token.child_token();

    tokio::spawn(async move { sensor_node.run(child).await });

    tokio::signal::ctrl_c()
        .await
        .inspect(|_| tracing::info!("Received Ctrl+C, shutting down..."))?;

    cancellation_token.cancel();

    Ok(())
}
