use crate::infrastructure::cli::Args;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    /// Name of the sensor
    pub name: String,

    /// Rate of telemetry messages per second
    pub rate: u32,

    /// Telemetry sink address (normalized to use correct scheme)
    pub telemetry_sink_address: String,

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
    fn normalize_sink_address(address: String, tls_enabled: bool) -> String {
        if tls_enabled {
            // TLS enabled: enforce HTTPS
            if let Some(stripped) = address.strip_prefix("http://") {
                let https_addr = format!("https://{stripped}");
                tracing::info!("Converted address to HTTPS for mTLS: {https_addr}");
                https_addr
            } else if !address.starts_with("https://") {
                let https_addr = format!("https://{address}");
                tracing::info!("Added HTTPS scheme for mTLS: {https_addr}");
                https_addr
            } else {
                address
            }
        } else {
            // TLS disabled: ensure HTTP or no scheme
            if address.starts_with("https://") {
                tracing::warn!(
                    "Address uses HTTPS scheme but TLS is not configured. This may cause connection issues."
                );
            }
            address
        }
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

        let tls_enabled = tls_config.is_some();
        let telemetry_sink_address = Self::normalize_sink_address(value.telemetry_sink_address, tls_enabled);

        Ok(Self {
            name: value.name,
            rate: value.rate,
            telemetry_sink_address,
            tls_config,
        })
    }
}
