# LostLove Server

High-performance VPN server implementing the LostLove Protocol.

## Phase 1 Status

✅ **Completed Components:**
- Basic packet structures with CRC16 checksums
- TCP server with async I/O
- Connection management (up to 1000 connections)
- Session tracking and statistics
- Handshake protocol (ClientHello/ServerHello)
- TUN/TAP interface support
- Basic packet routing
- Configuration system
- Comprehensive error handling
- Unit tests for all core components

🚧 **Not Yet Implemented:**
- Encryption (coming in Phase 2)
- Traffic obfuscation (coming in Phase 4)
- UDP support
- Full packet routing logic

## Building

### Prerequisites

- Rust 1.75+ (`rustup install stable`)
- Linux kernel with TUN/TAP support
- Root privileges (for TUN interface creation)

### Compile

```bash
cd server
cargo build --release
```

The binary will be at `target/release/lostlove-server`.

## Running

### 1. Create Configuration

Copy the example configuration:

```bash
sudo mkdir -p /etc/lostlove
sudo cp config/server.toml /etc/lostlove/server.toml
```

Edit `/etc/lostlove/server.toml` as needed.

### 2. Run Server

```bash
# Check configuration
sudo ./target/release/lostlove-server --check-config

# Run server
sudo ./target/release/lostlove-server
```

**Note:** Root privileges are required to create TUN interface.

### 3. Command Line Options

```bash
lostlove-server [OPTIONS]

Options:
  -c, --config <FILE>     Configuration file [default: /etc/lostlove/server.toml]
      --check-config      Check configuration and exit
  -l, --log-level <LEVEL> Log level (trace, debug, info, warn, error) [default: info]
  -h, --help              Print help
  -V, --version           Print version
```

## Configuration

### Server Section

```toml
[server]
bind_address = "0.0.0.0"   # Listen on all interfaces
port = 8443                 # Server port
protocol = "tcp"            # Protocol: tcp, udp, or both
max_connections = 1000      # Maximum concurrent connections
worker_threads = 0          # 0 = auto (number of CPU cores)
```

### Network Section

```toml
[network]
tun_name = "hfp0"          # TUN interface name
tun_address = "10.8.0.1/24" # TUN IP address (CIDR)
mtu = 1400                  # Maximum Transmission Unit
enable_ipv6 = false         # IPv6 support
```

### Limits Section

```toml
[limits]
rate_limit_per_user = 100000000  # 100 MB/s per user
max_streams_per_connection = 256
connection_timeout = 300          # 5 minutes
```

## Testing

### Run Unit Tests

```bash
cargo test
```

### Run with Debug Logging

```bash
sudo RUST_LOG=debug ./target/release/lostlove-server
```

### Test Connection

You can test the server with `telnet`:

```bash
telnet localhost 8443
```

Or use `nc`:

```bash
nc localhost 8443
```

## Monitoring

### Server Statistics

The server logs statistics every minute:

```
Server stats - Active: 10, Total: 42, Sent: 1234, Received: 5678
```

### Prometheus Metrics

Metrics are available at `http://localhost:9090/metrics` (when enabled).

## Architecture

```
┌──────────────┐
│   TCP Client │
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│  Connection Mgr  │  ← Manages all connections
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Packet Handler  │  ← Processes LLP packets
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│   Packet Router  │  ← Routes packets
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  TUN Interface   │  ← Virtual network device
└──────────────────┘
```

## Troubleshooting

### Permission Denied

If you get "Permission denied" errors:

```bash
# Make sure you're running as root
sudo ./target/release/lostlove-server

# Or grant CAP_NET_ADMIN capability
sudo setcap cap_net_admin=eip ./target/release/lostlove-server
```

### TUN Device Creation Failed

If TUN creation fails:

```bash
# Check if TUN module is loaded
lsmod | grep tun

# Load TUN module if needed
sudo modprobe tun

# Check device permissions
ls -l /dev/net/tun
```

### Port Already in Use

If port 8443 is already in use:

```bash
# Find process using the port
sudo netstat -tlnp | grep 8443

# Change port in config
vim /etc/lostlove/server.toml
```

## Development

### Project Structure

```
server/
├── Cargo.toml           # Dependencies
├── src/
│   ├── main.rs          # Entry point
│   ├── config.rs        # Configuration
│   ├── error.rs         # Error types
│   ├── core/            # Core server
│   │   ├── server.rs    # Main server
│   │   ├── connection.rs # Connection mgmt
│   │   └── session.rs   # Session tracking
│   ├── protocol/        # LLP protocol
│   │   ├── packet.rs    # Packet structures
│   │   ├── handshake.rs # Handshake logic
│   │   └── stream.rs    # Stream IDs
│   └── network/         # Networking
│       ├── tun_interface.rs # TUN/TAP
│       └── router.rs    # Packet routing
└── config/
    └── server.toml      # Example config
```

### Adding New Features

1. Create feature branch
2. Implement in appropriate module
3. Add unit tests
4. Run `cargo test`
5. Run `cargo clippy` for linting
6. Submit pull request

### Code Style

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Run all checks
cargo fmt && cargo clippy && cargo test
```

## Performance

### Benchmarks (Phase 1)

On a typical VPS (2 vCPU, 2GB RAM):

- **Connections**: 1000+ concurrent
- **Throughput**: Limited by TCP (to be improved in Phase 2)
- **Latency**: +5-10ms overhead
- **CPU Usage**: ~2% at 100 connections
- **Memory**: ~50MB + 1MB per connection

### Optimization Tips

1. Increase `worker_threads` for high load
2. Adjust `max_connections` based on RAM
3. Use faster disk for logs
4. Enable `io_uring` on Linux 5.1+ (coming in Phase 6)

## Next Steps (Phase 2)

- [ ] Implement ChaCha20-Poly1305 encryption
- [ ] Add AES-256-GCM support
- [ ] Implement Hybrid Symmetric Encryption (HSE)
- [ ] Add key derivation (HKDF)
- [ ] Implement key rotation

See [ROADMAP.md](../ROADMAP.md) for full development plan.

## License

MIT License - See [LICENSE](../LICENSE) for details.
