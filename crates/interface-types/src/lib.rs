#[derive(Debug, Clone)]
pub struct Telemetry {
    pub name: String,
    pub value: u32,
    pub timestamp: std::time::SystemTime,
}
