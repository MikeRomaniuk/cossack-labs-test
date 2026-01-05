use crate::infrastructure::cli::Args;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    /// Name of the sensor
    pub name: String,

    /// Rate of telemetry messages per second
    pub rate: u32,
}

impl Config {
    pub fn new(name: String, rate: u32) -> Self {
        Self { name, rate }
    }
}
