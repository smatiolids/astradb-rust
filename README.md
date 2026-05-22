# Astra DB Rust Driver - Connection Test

A Rust project demonstrating how to connect to DataStax Astra DB using the [cdrs-tokio](https://github.com/krojew/cdrs-tokio) async Cassandra driver.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Setup](#setup)
- [Running the Project](#running-the-project)
- [Project Structure](#project-structure)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements
- macOS, Linux, or Windows (with WSL2 recommended)
- Git
- Internet connection
- A DataStax Astra DB account and database instance

### Required Accounts
- [DataStax Astra DB](https://www.datastax.com/products/datastax-astra) - Free tier available

## Installation

### Step 1: Install Rust

#### On macOS/Linux:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then add Rust to your PATH:
```bash
source "$HOME/.cargo/env"
```

Verify installation:
```bash
rustc --version
cargo --version
```

#### On Windows (with WSL2):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then restart your WSL2 terminal and verify:
```bash
rustc --version
cargo --version
```

### Step 2: Install Required System Dependencies

#### On macOS (with Homebrew):
```bash
# Install OpenSSL for certificate handling
brew install openssl

# Install pkg-config (if not already installed)
brew install pkg-config
```

#### On Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install -y libssl-dev pkg-config build-essential
```

#### On Fedora/RHEL:
```bash
sudo dnf install -y openssl-devel pkg-config gcc
```

#### On Alpine:
```bash
apk add --no-cache openssl-dev musl-dev
```

### Step 3: Verify Rust Setup

```bash
# Check Rust version
rustc --version

# Check Cargo version
cargo --version

# Update Rust (optional, but recommended)
rustup update stable
```

## Setup

### Step 1: Clone or Navigate to the Project

```bash
cd /path/to/rust-drivers
```

### Step 2: Get Your Astra DB Credentials

1. Log in to [Astra DB Console](https://astra.datastax.com)
2. Select your database (or create a new one if needed)
3. Click the "Connect" tab
4. Copy your credentials:
   - **Database ID**: Found in the connection URL or dashboard
   - **Region**: Where your database is deployed (e.g., `us-east-1`)
   - **Client ID & Secret**: Generate new credentials under "Generate Token"

### Step 3: Create Environment Configuration

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env with your credentials
nano .env  # or use your preferred editor
```

Fill in the `.env` file with your Astra DB credentials:
```
ASTRA_DB_ID=your-actual-db-id
ASTRA_REGION=us-east-1
ASTRA_CLIENT_ID=your-actual-client-id
ASTRA_CLIENT_SECRET=your-actual-client-secret
```

**Important:** Never commit `.env` to version control. Use `.env.example` as a template.

### Step 4: Verify Certificate Authority (CA) Installation

The project requires system certificates for SSL/TLS connections:

#### macOS (Homebrew):
```bash
# Install OpenSSL if not already installed
brew install openssl

# Verify certificate path
ls -la /usr/local/etc/openssl/cert.pem
# or for M1/M2 Macs
ls -la /opt/homebrew/etc/openssl/cert.pem
```

#### Linux:
```bash
# Ubuntu/Debian
ls -la /etc/ssl/certs/ca-certificates.crt

# Fedora/RHEL
ls -la /etc/pki/tls/certs/ca-bundle.crt

# Alpine
ls -la /etc/ssl/cert.pem
```

The project will automatically detect the correct certificate path on startup.

## Running the Project

### Build the Project

```bash
# Download dependencies and compile
cargo build

# Build with optimizations (takes longer but runs faster)
cargo build --release
```

### Run the Application

```bash
# Development (debug) build
cargo run

# Release (optimized) build
cargo run --release
```

### Expected Output

On successful connection, you should see:
```
Connecting to Astra DB: your-db-id
Connecting to host: your-db-id-us-east-1.cassandra.datastax.com
Executing query: SELECT cluster_name, listen_address, partitioner FROM system.local
✓ Successfully connected to Astra DB
✓ Query executed successfully
Query result: {...cluster information...}
```

### Run Tests (if available)

```bash
cargo test
```

### Check Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy
```

## Project Structure

```
rust-drivers/
├── Cargo.toml              # Project manifest with dependencies
├── .env.example            # Example environment configuration
├── .gitignore              # Git ignore file
├── README.md               # This file
└── src/
    └── main.rs             # Main application code
```

## Configuration

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `ASTRA_DB_ID` | Your Astra DB identifier | `a1b2c3d4-e5f6-g7h8-i9j0-k1l2m3n4o5p6` |
| `ASTRA_REGION` | AWS region of your database | `us-east-1` |
| `ASTRA_CLIENT_ID` | Client ID for authentication | `YourClientId` |
| `ASTRA_CLIENT_SECRET` | Client secret for authentication | `YourClientSecret` |

### Dependencies

- **cdrs-tokio**: Async Cassandra driver for Rust
- **tokio**: Asynchronous runtime
- **anyhow**: Error handling
- **tracing**: Structured logging
- **serde**: Serialization/deserialization
- **dotenv**: Environment variable loading

## Troubleshooting

### Issue: "CA certificate not found"

**Solution:**
```bash
# macOS with Homebrew
brew install openssl

# Linux - Ubuntu/Debian
sudo apt-get install -y ca-certificates openssl

# Linux - Fedora/RHEL
sudo dnf install -y ca-certificates openssl

# Then run again
cargo run
```

### Issue: Connection timeout

**Solutions:**
1. Verify your database is running in the Astra console
2. Check internet connectivity: `ping google.com`
3. Verify credentials are correct in `.env`
4. Check firewall settings - port 9042 (Cassandra) must be accessible

### Issue: "ASTRA_DB_ID not set"

**Solution:**
```bash
# Ensure .env file exists and is in the project root
ls -la .env

# If missing, copy from example
cp .env.example .env

# Edit and fill in your credentials
nano .env
```

### Issue: "Connection refused" or "Host unreachable"

**Solutions:**
1. Verify the connection format:
   ```bash
   echo $ASTRA_DB_ID-$ASTRA_REGION.cassandra.datastax.com
   ```
2. Test DNS resolution:
   ```bash
   nslookup your-db-id-us-east-1.cassandra.datastax.com
   ```
3. Check your network connection and firewall settings

### Issue: Build fails with "Package not found"

**Solution:**
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

### Issue: Cargo command not found

**Solution:**
```bash
# Ensure Rust is in your PATH
source "$HOME/.cargo/env"

# Verify installation
rustc --version
```

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/)
- [cdrs-tokio on GitHub](https://github.com/krojew/cdrs-tokio)
- [DataStax Astra DB Documentation](https://docs.datastax.com/en/astra/docs/)
- [Cassandra Query Language (CQL) Reference](https://cassandra.apache.org/doc/latest/cassandra/cql/)

## Development Tips

### Running with Debug Logs

```bash
RUST_LOG=debug cargo run
```

### Building for Production

```bash
# Build optimized release binary
cargo build --release

# Run the optimized binary
./target/release/astra-db-rust
```

### Profiling and Benchmarking

```bash
# Check compilation time
cargo build --timings
```

## License

This project is provided as-is for educational and testing purposes.

## Support

For issues with:
- **cdrs-tokio driver**: See [GitHub Issues](https://github.com/krojew/cdrs-tokio/issues)
- **Astra DB**: Visit [DataStax Support](https://support.datastax.com/)
- **Rust**: Visit [Rust Forums](https://users.rust-lang.org/)
