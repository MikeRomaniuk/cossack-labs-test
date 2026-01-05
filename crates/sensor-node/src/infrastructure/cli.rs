use core::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
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
}
