use interface_types::Telemetry;

mod file_logger;
mod grpc;

pub use file_logger::FileLogger;
pub use grpc::{TelemetryServer, TelemetryService};
