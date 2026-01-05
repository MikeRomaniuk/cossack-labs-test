pub mod service {
    pub mod telemetry {
        tonic::include_proto!("telemetry");

        impl From<interface_types::Telemetry> for SensorTelemetry {
            fn from(value: interface_types::Telemetry) -> Self {
                Self {
                    name: value.name,
                    value: value.value,
                    timestamp: Some(prost_types::Timestamp::from(value.timestamp)),
                }
            }
        }

        impl TryFrom<SensorTelemetry> for interface_types::Telemetry {
            type Error = String;

            fn try_from(value: SensorTelemetry) -> std::result::Result<Self, Self::Error> {
                let timestamp = value.timestamp.ok_or("missing timestamp".to_string())?;
                let timestamp =
                    std::time::SystemTime::try_from(timestamp).map_err(|_| "invalid timestamp".to_string())?;

                Ok(Self {
                    name: value.name,
                    value: value.value,
                    timestamp,
                })
            }
        }
    }
}
