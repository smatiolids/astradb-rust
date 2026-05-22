# Quick Start Guide

Get up and running with the Astra DB Rust driver in 5 minutes.

## 1. Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## 2. Install System Dependencies

**macOS:**
```bash
brew install openssl pkg-config
```

**Ubuntu/Debian:**
```bash
sudo apt-get update && sudo apt-get install -y libssl-dev pkg-config
```

**Fedora/RHEL:**
```bash
sudo dnf install -y openssl-devel pkg-config
```

## 3. Get Your Astra DB Credentials

1. Log in to [Astra DB Console](https://astra.datastax.com)
2. Click "Connect" on your database
3. Copy: DB ID, Region, Client ID, and Client Secret
4. Create `.env` file in this directory:

```bash
cp .env.example .env
nano .env
# Fill in your credentials
```

## 4. Build and Run

```bash
# Build the project
cargo build

# Run the application
cargo run
```

## Success!

You should see:
```
✓ Successfully connected to Astra DB
✓ Query executed successfully
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `rustc: command not found` | Run: `source "$HOME/.cargo/env"` |
| `CA certificate not found` | macOS: `brew install openssl` |
| `ASTRA_DB_ID not set` | Make sure `.env` file is created and filled |
| Connection timeout | Verify credentials and internet connection |

For more details, see [README.md](README.md).
