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
    pub cert: String,

    /// Path to the client private key file
    pub key: String,

    /// Path to the CA certificate file for server verification
    pub ca: String,
}

impl Config {
    pub fn new(name: String, rate: u32, tls_config: Option<TlsConfig>) -> Self {
        Self { name, rate, tls_config }
    }
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
            name: value.name,
            rate: value.rate,
            tls_config,
        })
    }
}
