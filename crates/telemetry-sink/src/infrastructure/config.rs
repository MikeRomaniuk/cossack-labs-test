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
    /// Server certificate file
    pub cert: String,

    /// Server private key file
    pub key: String,

    /// CA certificate file for client verification
    pub ca: String,
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;
    fn try_from(value: Args) -> Result<Self, Self::Error> {
        let tls_config = match (value.tls_cert, value.tls_key, value.tls_ca) {
            (Some(cert), Some(key), Some(ca)) => {
                let server_root_ca_cert = std::fs::read_to_string(&ca)?;
                let client_cert = std::fs::read_to_string(&cert)?;
                let client_key = std::fs::read_to_string(&key)?;

                Some(TlsConfig {
                    cert: client_cert,
                    key: client_key,
                    ca: server_root_ca_cert,
                })
            }
            _ => None,
        };

        Ok(Self {
            ip: value.ip,
            log_file: value.log_file,
            buffer_size: value.buffer_size,
            buffer_flush_interval: value.buffer_flush_interval,
            rate_limit: value.rate_limit,
            tls_config,
        })
    }
}
