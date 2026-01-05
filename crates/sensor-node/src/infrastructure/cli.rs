use core::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Clone)]
#[clap(author = "Mykhailo Romaniuk", about = "Sensor Node")]
#[clap(version, long_about = None)]
pub(crate) struct Args {
    /// Address of the telemetry sink
    #[clap(short, long, value_parser, default_value = "http://127.0.0.1:3000")]
    pub telemetry_sink_address: String,

    /// Name of the sensor
    #[clap(short, long, default_value = "uninitialized")]
    pub name: String,

    /// Rate of telemetry messages per second
    #[clap(short, long, default_value = "10")]
    pub rate: u32,

    /// Number of worker threads to use for telemetry processing.
    ///
    /// Defaults to the number of cores available to the system.
    #[clap(short, long)]
    pub worker_threads: Option<usize>,

    /// Path to the client certificate file (PEM format) for mTLS
    #[clap(long)]
    pub tls_cert: Option<PathBuf>,

    /// Path to the client private key file (PEM format) for mTLS
    #[clap(long)]
    pub tls_key: Option<PathBuf>,

    /// Path to the CA certificate file (PEM format) for server verification
    #[clap(long)]
    pub tls_ca: Option<PathBuf>,
}
