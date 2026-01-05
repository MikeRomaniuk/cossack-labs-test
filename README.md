# Distributed Telemetry System

Test task of Cossack Labs to Romaniuk Mykhailo.

## Task completion 

Basic:

- [x] Component 1: Sensor node implementation
- [x] Component 2: Telemetry sink implementation

Bonus:

- [x] gRPC communication between nodes
- [x] Support for multiple sensor sessions
- [x] mTLS support for authenticated communication
- [x] Graceful shutdown via `Ctrl+C`
- [x] Encrypt each message

#### Note

It was required to rate limit the telemetry stream in bytes/sec. I have done the rate limiting in message/sec, 
because of the nature of `tonic` crate which abstracts away the bytes and works with the messages directly.
I think that it does not matter much, because the rate limit is still implemented. 

### Layered Architecture Pattern

Both binaries follow a clean architecture with strict layer separation:

#### 1. Domain Layer (`domain/`)
Core business logic, framework-agnostic and independent of external systems.

- **sensor-node**: `SensorNode` generates telemetry at configurable rates
- **telemetry-sink**: `TelemetryService` manages buffering with rate-limited queuing via `RateLimitedQueue`

#### 2. Adapter Layer (`adapter/`)
Implements external communication using concrete implementations of trait interfaces.

- **sensor-node**: gRPC client adapter implementing `TelemetryAdapter` trait
- **telemetry-sink**: gRPC server adapter and file logger implementing `TelemetryReceiver` and `LogWriter<T>` traits

#### 3. Infrastructure Layer (`infrastructure/`)
Configuration, CLI parsing, and application setup.

- `cli.rs`: Command-line argument parsing using Clap
- `config.rs`: Application configuration structs with validation and normalization

#### 4. Interface Layer (`interface/`)
Trait definitions for dependency inversion (telemetry-sink only).

- `TelemetryReceiver`: Trait for receiving telemetry streams
- `LogWriter<T>`: Trait for writing logs

### Key Architectural Principles

- **Dependency Inversion**: Domain logic depends only on traits, not concrete implementations
- **Dependency Injection**: Adapters are injected into domain services at runtime
- **Boundary Translation**: gRPC types are converted to/from domain types at adapter boundaries
- **Graceful Shutdown**: Cancellation tokens propagate shutdown signals throughout the system

### Shared Crates

- **interface-types**: Common domain types shared between binaries (e.g., `Telemetry` struct)
- **grpc-types**: Auto-generated gRPC code from protobuf definitions (`grpc/telemetry.proto`)

## Technology Stack

### Core Libraries

| Library | Purpose | Usage |
|---------|---------|-------|
| **tokio** | Async runtime | Powers all async operations, provides multi-threaded executor |
| **tonic** | gRPC framework | Client/server communication, TLS support |
| **prost** | Protocol Buffers | Serialization for gRPC messages |
| **clap** | CLI parsing | Command-line argument handling with derive macros |
| **tracing** | Structured logging | Configurable logging with environment variables |
| **anyhow** | Error handling | Ergonomic error propagation and context |
| **tokio-util** | Async utilities | Cancellation tokens for graceful shutdown |

## Getting Started

### Prerequisites

- Rust 1.92.0 or later (pinned in `rust-toolchain.toml`)
- Protocol Buffers compiler (for regenerating gRPC code, optional)

### Building the Project

```bash
# Build entire workspace
cargo build

# Build with optimizations (recommended for production)
cargo build --release

# Build specific binary
cargo build --bin sensor-node
cargo build --bin telemetry-sink
```

### Running the System

#### 1. Start the Telemetry Sink

The sink must be running before starting sensor nodes.

```bash
# Default configuration (listens on 127.0.0.1:3000)
cargo run --bin telemetry-sink

# Custom configuration
cargo run --bin telemetry-sink -- \
  --ip 127.0.0.1:3000 \
  --log-file ./telemetry.log \
  --buffer-size 100 \
  --rate-limit 200
```

**Telemetry Sink Options:**
- `--ip`: Bind address (default: `127.0.0.1:3000`)
- `--log-file`: Output log file path (default: `./output.log`)
- `--buffer-size`: Buffer size before flush (default: `50`)
- `--rate-limit`: Max messages per second (default: `100`)
- `--tls-cert`: Server certificate file for TLS
- `--tls-key`: Server private key file for TLS
- `--tls-ca`: CA certificate for client verification (mTLS)

#### 2. Start Sensor Nodes

```bash
# Default configuration (connects to http://127.0.0.1:3000)
cargo run --bin sensor-node

# Custom configuration
cargo run --bin sensor-node -- \
  --name my-sensor \
  --rate 20 \
  --telemetry-sink-address http://127.0.0.1:3000
```

**Sensor Node Options:**
- `--telemetry-sink-address` / `-t`: Sink address (default: `http://127.0.0.1:3000`)
- `--name` / `-n`: Sensor identifier (default: `uninitialized`)
- `--rate` / `-r`: Messages per second (default: `10`)
- `--worker-threads` / `-w`: Tokio worker threads (default: number of cores)
- `--tls-cert`: Client certificate file for mTLS
- `--tls-key`: Client private key file for mTLS
- `--tls-ca`: CA certificate for server verification

### Running with mTLS

Generate certificates: you can generate self-signed certificates using `openssl` via the `data/tls/create.sh` script.

Start with mTLS enabled:

```bash
# Start sink with mTLS
cargo run --bin telemetry-sink -- \
  --tls-cert server-cert.pem \
  --tls-key server-key.pem \
  --tls-ca ca-cert.pem

# Start sensor with mTLS
cargo run --bin sensor-node -- \
  --telemetry-sink-address 127.0.0.1:3000 \
  --tls-cert client-cert.pem \
  --tls-key client-key.pem \
  --tls-ca ca-cert.pem
```

**Note**: When TLS is enabled, the sensor node automatically converts addresses to use the `https://` scheme.

### Logging Configuration

Both binaries use the `tracing` crate with environment variable configuration:

```bash
# Set log level for sensor node (default: INFO)
SENSOR_NODE_LOG=debug cargo run --bin sensor-node

# Set log level for telemetry sink (default: INFO)
TELEMETRY_SINK_LOG=trace cargo run --bin telemetry-sink

# Available levels: trace, debug, info, warn, error
```

### Graceful Shutdown

Both binaries support graceful shutdown via `Ctrl+C`:
- Sensor nodes stop generating telemetry and close streams
- Telemetry sink flushes remaining buffers and closes connections

### Regenerating gRPC Code

The `grpc-types` crate automatically regenerates Rust code from `grpc/telemetry.proto` via build script:

```bash
cargo build --package grpc-types
```

## Workspace Configuration

- **Rust Edition**: 2024
- **Toolchain**: 1.92.0 (pinned)
- **Linting**: Extremely strict workspace lints (see `Cargo.toml`)
  - No `unwrap()`, `panic!()`, or unchecked `as` conversions
  - Extensive correctness and style checks
  - All crates inherit via `[lints] workspace = true`

## Author

Mykhailo Romaniuk
