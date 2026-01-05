use std::path::PathBuf;

use crate::infrastructure::cli::Args;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    /// Name of the sensor
    pub name: String,

    /// Rate of telemetry messages per second
    pub rate: u32,

    /// mTLS configuration
    pub tls_config: Option<TlsConfig>,
}

#[derive(Debug, Clone)]
pub(crate) struct TlsConfig {
    /// Path to the client certificate file
    pub cert: PathBuf,

    /// Path to the client private key file
    pub key: PathBuf,

    /// Path to the CA certificate file for server verification
    pub ca: PathBuf,
}

impl Config {
    pub fn new(name: String, rate: u32, tls_config: Option<TlsConfig>) -> Self {
        Self {
            name,
            rate,
            tls_config,
        }
    }
}

impl From<Args> for Config {
    fn from(value: Args) -> Self {
        let tls_config = match (value.tls_cert, value.tls_key, value.tls_ca) {
            (Some(cert), Some(key), Some(ca)) => Some(TlsConfig { cert, key, ca }),
            _ => None,
        };

        Self {
            name: value.name,
            rate: value.rate,
            tls_config,
        }
    }
}
