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
}

impl From<Args> for Config {
    fn from(value: Args) -> Self {
        Self {
            ip: value.ip,
            log_file: value.log_file,
            buffer_size: value.buffer_size,
            buffer_flush_interval: value.buffer_flush_interval,
            rate_limit: value.rate_limit,
        }
    }
}
