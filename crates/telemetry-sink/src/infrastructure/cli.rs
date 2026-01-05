use core::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author = "Mykhailo Romaniuk", about = "Telemetry Sink")]
#[clap(version, long_about = None)]
pub(crate) struct Args {
    /// Address of the telemetry sink
    #[clap(short, long, value_parser, default_value = "127.0.0.1:3000")]
    pub ip: SocketAddr,

    /// Path to the log file
    #[clap(short, long, value_parser, default_value = "./telemetry.log")]
    pub log_file: PathBuf,

    /// Size of the in-memory buffer for telemetry messages
    #[clap(short, long, default_value = "10")]
    pub buffer_size: usize,

    /// Interval between flushes of the in-memory buffer
    #[clap(long, value_parser=milliseconds_parser, default_value = "100")]
    pub buffer_flush_interval: std::time::Duration,

    /// Maximum number of messages to receive per second
    #[clap(long, short, default_value = "100")]
    pub rate_limit: usize,

    /// Number of worker threads to use for telemetry processing.
    ///
    /// Defaults to the number of cores available to the system.
    #[clap(short, long)]
    pub worker_threads: Option<usize>,
}

fn milliseconds_parser(s: &str) -> Result<std::time::Duration, String> {
    s.parse::<u64>()
        .ok()
        .filter(|&n| n > 0)
        .map(std::time::Duration::from_millis)
        .ok_or_else(|| "seconds must be a positive integer".to_owned())
}
