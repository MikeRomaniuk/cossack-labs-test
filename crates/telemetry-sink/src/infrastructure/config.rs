use core::net::SocketAddr;
use std::path::PathBuf;

use crate::infrastructure::cli::Args;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    /// Address to bind the gRPC server to
    pub ip: SocketAddr,

    /// Path to the log file
    pub log_file: PathBuf,

    /// Size of the in-memory buffer for telemetry messages
    pub buffer_size: usize,

    /// Interval between flushes of the in-memory buffer
    pub buffer_flush_interval: std::time::Duration,

    /// Maximum number of messages to receive per second
    pub rate_limit: usize,

    /// mTLS configuration
    pub tls_config: Option<TlsConfig>,
}

#[derive(Debug, Clone)]
pub(crate) struct TlsConfig {
    /// Path to the server certificate file
    pub cert: PathBuf,

    /// Path to the server private key file
    pub key: PathBuf,

    /// Path to the CA certificate file for client verification
    pub ca: PathBuf,
}

impl From<Args> for Config {
    fn from(value: Args) -> Self {
        let tls_config = match (value.tls_cert, value.tls_key, value.tls_ca) {
            (Some(cert), Some(key), Some(ca)) => Some(TlsConfig { cert, key, ca }),
            _ => None,
        };

        Self {
            ip: value.ip,
            log_file: value.log_file,
            buffer_size: value.buffer_size,
            buffer_flush_interval: value.buffer_flush_interval,
            rate_limit: value.rate_limit,
            tls_config,
        }
    }
}
