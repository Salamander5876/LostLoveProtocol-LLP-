# LostLove Server - Архитектура

## Обзор

LostLove Server - это высокопроизводительный VPN сервер, написанный на Rust, обеспечивающий безопасное туннелирование с продвинутой обфускацией.

## Архитектура системы

```
┌────────────────────────────────────────────────────────┐
│                    Load Balancer                       │
│              (Optional HAProxy/Nginx)                  │
└────────────────────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────┐
│                  LostLove Server                      │
│  ┌──────────────────────────────────────────────────┐ │
│  │           Control Plane                          │ │
│  │  ┌─────────────────────────────────────────────┐ │ │
│  │  │  - Authentication                           │ │ │
│  │  │  - Rate Limiting                            │ │ │
│  │  │  - Statistics & Monitoring                  │ │ │
│  │  │  - Configuration Management                 │ │ │
│  │  └─────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────┐ │
│  │           Data Plane                             │ │
│  │  ┌─────────────────────────────────────────────┐ │ │
│  │  │  Ingress → Processing → Egress              │ │ │
│  │  │    ↓          ↓           ↓                 │ │ │
│  │  │  Decrypt   Route      Encrypt               │ │ │
│  │  └─────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────┐ │
│  │           Core Engine (Rust)                     │ │
│  │  ┌─────────────────────────────────────────────┐ │ │
│  │  │  - Connection Manager                       │ │ │
│  │  │  - Crypto Engine (QuantumShield)            │ │ │
│  │  │  - Packet Router                            │ │ │
│  │  │  - Memory Pool                              │ │ │
│  │  └─────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────┐
│                  System Layer                          │
│  ┌──────────────────────────────────────────────────┐ │
│  │  - TUN/TAP Interface                             │ │
│  │  - Kernel Networking Stack                       │ │
│  │  - iptables/nftables Rules                       │ │
│  └──────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
```

## Структура проекта

```
server/
├── src/
│   ├── main.rs                    # Entry point
│   ├── lib.rs                     # Library exports
│   │
│   ├── core/                      # Ядро сервера
│   │   ├── mod.rs
│   │   ├── server.rs              # Главный сервер
│   │   ├── connection.rs          # Управление соединениями
│   │   ├── session.rs             # Управление сессиями
│   │   └── thread_pool.rs         # Пул потоков
│   │
│   ├── protocol/                  # Протокол LLP
│   │   ├── mod.rs
│   │   ├── packet.rs              # Структуры пакетов
│   │   ├── handshake.rs           # Handshake логика
│   │   ├── stream.rs              # Мультиплексирование
│   │   └── codec.rs               # Кодирование/декодирование
│   │
│   ├── crypto/                    # Криптография
│   │   ├── mod.rs
│   │   ├── quantum_shield.rs      # QuantumShield implementation
│   │   ├── ecc.rs                 # Dynamic ECC
│   │   ├── hse.rs                 # Hybrid Symmetric Encryption
│   │   ├── qrl.rs                 # Quantum Resistant Layer
│   │   └── key_manager.rs         # Управление ключами
│   │
│   ├── obfuscation/               # Обфускация трафика
│   │   ├── mod.rs
│   │   ├── chameleon.rs           # Chameleon Disguise System
│   │   ├── traffic_shaper.rs      # Traffic shaping
│   │   ├── dpi_evasion.rs         # DPI evasion
│   │   └── domain_fronting.rs     # Domain fronting
│   │
│   ├── network/                   # Сетевой слой
│   │   ├── mod.rs
│   │   ├── tun.rs                 # TUN interface
│   │   ├── router.rs              # Пакетная маршрутизация
│   │   ├── nat.rs                 # NAT traversal
│   │   └── firewall.rs            # Firewall rules
│   │
│   ├── auth/                      # Аутентификация
│   │   ├── mod.rs
│   │   ├── user_manager.rs        # Управление пользователями
│   │   ├── token.rs               # JWT токены
│   │   └── rate_limiter.rs        # Rate limiting
│   │
│   ├── storage/                   # Хранилище данных
│   │   ├── mod.rs
│   │   ├── database.rs            # SQLite/PostgreSQL
│   │   └── cache.rs               # Redis cache
│   │
│   ├── api/                       # API сервер
│   │   ├── mod.rs
│   │   ├── admin.rs               # Admin API
│   │   ├── metrics.rs             # Metrics API
│   │   └── webhooks.rs            # Webhooks
│   │
│   └── utils/                     # Утилиты
│       ├── mod.rs
│       ├── config.rs              # Конфигурация
│       ├── logger.rs              # Логирование
│       └── metrics.rs             # Метрики
│
├── config/
│   ├── server.toml                # Основная конфигурация
│   ├── users.toml                 # Пользователи
│   └── obfuscation.toml           # Настройки обфускации
│
├── scripts/
│   ├── install.sh                 # Скрипт установки
│   ├── uninstall.sh               # Скрипт удаления
│   └── manage.sh                  # Управление сервером
│
├── systemd/
│   └── lostlove-server.service   # Systemd unit
│
├── Cargo.toml                     # Rust dependencies
└── README.md
```

## Core Engine

### 1. Server Main Loop

```rust
// src/core/server.rs

use tokio::runtime::Runtime;
use std::sync::Arc;

pub struct LostLoveServer {
    config: Arc<ServerConfig>,
    connection_manager: Arc<ConnectionManager>,
    crypto_engine: Arc<CryptoEngine>,
    obfuscation_engine: Arc<ObfuscationEngine>,
    thread_pool: ThreadPool,
}

impl LostLoveServer {
    pub fn new(config: ServerConfig) -> Result<Self> {
        let config = Arc::new(config);

        let connection_manager = Arc::new(
            ConnectionManager::new(config.max_connections)
        );

        let crypto_engine = Arc::new(
            CryptoEngine::new(&config.crypto)
        );

        let obfuscation_engine = Arc::new(
            ObfuscationEngine::new(&config.obfuscation)
        );

        let thread_pool = ThreadPool::new(
            num_cpus::get() * 2
        );

        Ok(Self {
            config,
            connection_manager,
            crypto_engine,
            obfuscation_engine,
            thread_pool,
        })
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting LostLove Server v{}", env!("CARGO_PKG_VERSION"));

        // 1. Initialize TUN interface
        let tun = self.setup_tun_interface().await?;
        info!("TUN interface created: {}", tun.name());

        // 2. Setup firewall rules
        self.setup_firewall_rules().await?;

        // 3. Start listeners
        let (tcp_listener, udp_socket) = self.start_listeners().await?;

        // 4. Start background tasks
        self.start_background_tasks().await?;

        // 5. Main event loop
        self.event_loop(tcp_listener, udp_socket, tun).await?;

        Ok(())
    }

    async fn event_loop(
        &self,
        tcp_listener: TcpListener,
        udp_socket: UdpSocket,
        tun: TunInterface,
    ) -> Result<()> {
        let mut tcp_incoming = tcp_listener.incoming();
        let mut udp_buffer = vec![0u8; 65536];

        loop {
            tokio::select! {
                // TCP connections (for initial handshake)
                Some(stream) = tcp_incoming.next() => {
                    let stream = stream?;
                    self.handle_new_connection(stream).await?;
                }

                // UDP packets (main data channel)
                Ok((len, addr)) = udp_socket.recv_from(&mut udp_buffer) => {
                    let packet = &udp_buffer[..len];
                    self.handle_udp_packet(packet, addr).await?;
                }

                // TUN interface (packets from system)
                Ok(packet) = tun.recv() => {
                    self.handle_tun_packet(packet).await?;
                }
            }
        }
    }
}
```

### 2. Connection Manager

```rust
// src/core/connection.rs

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ConnectionManager {
    connections: DashMap<SessionId, Connection>,
    next_session_id: AtomicU64,
    max_connections: usize,
}

impl ConnectionManager {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: DashMap::new(),
            next_session_id: AtomicU64::new(1),
            max_connections,
        }
    }

    pub fn create_session(&self, peer_addr: SocketAddr) -> Result<SessionId> {
        if self.connections.len() >= self.max_connections {
            return Err(Error::TooManyConnections);
        }

        let session_id = SessionId(
            self.next_session_id.fetch_add(1, Ordering::SeqCst)
        );

        let connection = Connection::new(session_id, peer_addr);
        self.connections.insert(session_id, connection);

        info!("New session created: {} from {}", session_id, peer_addr);
        Ok(session_id)
    }

    pub fn get(&self, session_id: &SessionId) -> Option<ConnectionRef> {
        self.connections.get(session_id).map(|c| ConnectionRef(c))
    }

    pub fn remove(&self, session_id: &SessionId) -> Option<Connection> {
        self.connections.remove(session_id).map(|(_, c)| c)
    }

    pub fn active_count(&self) -> usize {
        self.connections.len()
    }

    pub fn cleanup_stale(&self, timeout: Duration) {
        let now = Instant::now();
        self.connections.retain(|_, conn| {
            now.duration_since(conn.last_activity()) < timeout
        });
    }
}

pub struct Connection {
    session_id: SessionId,
    peer_addr: SocketAddr,
    state: Arc<RwLock<ConnectionState>>,
    crypto_state: Arc<RwLock<CryptoState>>,
    stats: ConnectionStats,
    last_activity: Arc<Mutex<Instant>>,
    streams: DashMap<StreamId, Stream>,
}

impl Connection {
    fn new(session_id: SessionId, peer_addr: SocketAddr) -> Self {
        Self {
            session_id,
            peer_addr,
            state: Arc::new(RwLock::new(ConnectionState::Handshaking)),
            crypto_state: Arc::new(RwLock::new(CryptoState::default())),
            stats: ConnectionStats::default(),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            streams: DashMap::new(),
        }
    }

    pub fn update_activity(&self) {
        *self.last_activity.lock() = Instant::now();
    }

    pub async fn send_packet(&self, packet: Packet) -> Result<()> {
        self.update_activity();
        self.stats.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.stats.bytes_sent.fetch_add(packet.len() as u64, Ordering::Relaxed);

        // Send through appropriate channel
        match self.state.read().await.channel_type() {
            ChannelType::Tcp => self.send_tcp(packet).await,
            ChannelType::Udp => self.send_udp(packet).await,
        }
    }
}

#[derive(Default)]
pub struct ConnectionStats {
    pub packets_sent: AtomicU64,
    pub packets_received: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub errors: AtomicU64,
}
```

### 3. Crypto Engine

```rust
// src/crypto/quantum_shield.rs

pub struct CryptoEngine {
    config: CryptoConfig,
    key_manager: Arc<KeyManager>,
}

impl CryptoEngine {
    pub fn new(config: &CryptoConfig) -> Self {
        Self {
            config: config.clone(),
            key_manager: Arc::new(KeyManager::new()),
        }
    }

    pub async fn encrypt(&self, plaintext: &[u8], session: &Session) -> Result<Vec<u8>> {
        let keys = self.key_manager.get_session_keys(session.id())?;

        // Layer 3: QRL (innermost)
        let layer3 = if self.config.qrl_enabled {
            self.qrl_encrypt(plaintext, &keys.qrl_key).await?
        } else {
            plaintext.to_vec()
        };

        // Layer 2: HSE (middle)
        let layer2 = if self.config.hse_enabled {
            self.hse_encrypt(&layer3, &keys.chacha_key, &keys.aes_key).await?
        } else {
            layer3
        };

        // Layer 1: ECC (outer)
        let layer1 = if self.config.ecc_enabled {
            self.ecc_encrypt(&layer2, &keys.ecc_public).await?
        } else {
            layer2
        };

        Ok(layer1)
    }

    pub async fn decrypt(&self, ciphertext: &[u8], session: &Session) -> Result<Vec<u8>> {
        let keys = self.key_manager.get_session_keys(session.id())?;

        // Layer 1: ECC (outer)
        let layer2 = if self.config.ecc_enabled {
            self.ecc_decrypt(ciphertext, &keys.ecc_private).await?
        } else {
            ciphertext.to_vec()
        };

        // Layer 2: HSE (middle)
        let layer3 = if self.config.hse_enabled {
            self.hse_decrypt(&layer2, &keys.chacha_key, &keys.aes_key).await?
        } else {
            layer2
        };

        // Layer 3: QRL (innermost)
        let plaintext = if self.config.qrl_enabled {
            self.qrl_decrypt(&layer3, &keys.qrl_key).await?
        } else {
            layer3
        };

        Ok(plaintext)
    }

    async fn hse_encrypt(
        &self,
        plaintext: &[u8],
        chacha_key: &ChaChaKey,
        aes_key: &AesKey,
    ) -> Result<Vec<u8>> {
        // Parallel encryption
        let (c1, c2) = tokio::join!(
            self.chacha20_encrypt(plaintext, chacha_key),
            self.aes512_encrypt(plaintext, aes_key)
        );

        let c1 = c1?;
        let c2 = c2?;

        // XOR results
        let mut result = Vec::with_capacity(c1.len());
        for (b1, b2) in c1.iter().zip(c2.iter()) {
            result.push(b1 ^ b2);
        }

        Ok(result)
    }
}
```

### 4. Packet Processing Pipeline

```rust
// src/protocol/packet.rs

pub struct PacketProcessor {
    crypto: Arc<CryptoEngine>,
    obfuscation: Arc<ObfuscationEngine>,
    router: Arc<PacketRouter>,
}

impl PacketProcessor {
    pub async fn process_incoming(&self, raw_data: &[u8], conn: &Connection) -> Result<()> {
        // 1. De-obfuscate
        let deobfuscated = self.obfuscation.deobfuscate(raw_data, conn).await?;

        // 2. Parse packet
        let packet = Packet::parse(&deobfuscated)?;

        // 3. Validate
        self.validate_packet(&packet, conn)?;

        // 4. Decrypt
        let plaintext = self.crypto.decrypt(&packet.payload, conn.session()).await?;

        // 5. Route based on packet type
        match packet.packet_type {
            PacketType::Data => {
                self.router.route_data(plaintext, conn).await?;
            }
            PacketType::Ack => {
                conn.handle_ack(&packet)?;
            }
            PacketType::KeepAlive => {
                conn.update_activity();
            }
            PacketType::Disconnect => {
                conn.close().await?;
            }
            _ => {
                warn!("Unknown packet type: {:?}", packet.packet_type);
            }
        }

        Ok(())
    }

    pub async fn process_outgoing(&self, data: &[u8], conn: &Connection) -> Result<Vec<u8>> {
        // 1. Create packet
        let packet = Packet::new(PacketType::Data, data);

        // 2. Encrypt
        let encrypted = self.crypto.encrypt(&packet.serialize(), conn.session()).await?;

        // 3. Obfuscate
        let obfuscated = self.obfuscation.obfuscate(&encrypted, conn).await?;

        Ok(obfuscated)
    }

    fn validate_packet(&self, packet: &Packet, conn: &Connection) -> Result<()> {
        // Check sequence number
        if !conn.is_valid_sequence(packet.sequence_number) {
            return Err(Error::InvalidSequence);
        }

        // Check timestamp (±30 seconds)
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let timestamp_diff = (now as i64 - packet.timestamp as i64).abs();
        if timestamp_diff > 30_000 {
            return Err(Error::TimestampTooOld);
        }

        // Verify checksum
        if !packet.verify_checksum() {
            return Err(Error::ChecksumMismatch);
        }

        Ok(())
    }
}
```

## Performance Optimizations

### 1. Zero-Copy I/O

```rust
// src/network/tun.rs

use io_uring::{opcode, types, IoUring};

pub struct TunInterface {
    fd: RawFd,
    ring: IoUring,
    buffer_pool: Arc<BufferPool>,
}

impl TunInterface {
    pub async fn recv_zero_copy(&mut self) -> Result<Bytes> {
        // Get buffer from pool (no allocation)
        let mut buf = self.buffer_pool.acquire();

        // Submit read operation to io_uring
        let read_op = opcode::Read::new(
            types::Fd(self.fd),
            buf.as_mut_ptr(),
            buf.len() as u32
        );

        unsafe {
            self.ring.submission()
                .push(&read_op.build())?;
        }

        self.ring.submit_and_wait(1)?;

        // Get completion
        let cqe = self.ring.completion().next().unwrap();
        let bytes_read = cqe.result() as usize;

        buf.truncate(bytes_read);
        Ok(buf.freeze())
    }
}

pub struct BufferPool {
    pool: ArrayQueue<Vec<u8>>,
    buffer_size: usize,
}

impl BufferPool {
    pub fn new(size: usize, buffer_size: usize) -> Self {
        let pool = ArrayQueue::new(size);
        for _ in 0..size {
            pool.push(vec![0u8; buffer_size]).ok();
        }

        Self { pool, buffer_size }
    }

    pub fn acquire(&self) -> Vec<u8> {
        self.pool.pop()
            .unwrap_or_else(|| vec![0u8; self.buffer_size])
    }

    pub fn release(&self, mut buf: Vec<u8>) {
        buf.clear();
        buf.resize(self.buffer_size, 0);
        self.pool.push(buf).ok();
    }
}
```

### 2. Lock-Free Structures

```rust
// src/core/session.rs

use crossbeam::queue::SegQueue;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

pub struct LockFreeSession {
    id: SessionId,
    send_queue: SegQueue<Packet>,
    recv_queue: SegQueue<Packet>,
    sequence_number: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    active: AtomicBool,
}

impl LockFreeSession {
    pub fn enqueue_send(&self, packet: Packet) {
        self.send_queue.push(packet);
    }

    pub fn dequeue_send(&self) -> Option<Packet> {
        self.send_queue.pop()
    }

    pub fn next_sequence(&self) -> u64 {
        self.sequence_number.fetch_add(1, Ordering::SeqCst)
    }
}
```

### 3. Parallel Processing

```rust
// src/core/thread_pool.rs

use rayon::prelude::*;

pub struct ThreadPool {
    pool: rayon::ThreadPool,
}

impl ThreadPool {
    pub fn process_batch(&self, packets: Vec<Packet>) -> Vec<Result<ProcessedPacket>> {
        // Process packets in parallel
        packets.par_iter()
            .map(|packet| self.process_single(packet))
            .collect()
    }

    fn process_single(&self, packet: &Packet) -> Result<ProcessedPacket> {
        // CPU-intensive operations (encryption/decryption)
        // are distributed across threads
        todo!()
    }
}
```

## Monitoring & Metrics

```rust
// src/utils/metrics.rs

use prometheus::{Counter, Gauge, Histogram};

pub struct ServerMetrics {
    pub active_connections: Gauge,
    pub packets_processed: Counter,
    pub bytes_transferred: Counter,
    pub processing_time: Histogram,
    pub errors: Counter,
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self {
            active_connections: Gauge::new("active_connections", "Active connections").unwrap(),
            packets_processed: Counter::new("packets_processed", "Packets processed").unwrap(),
            bytes_transferred: Counter::new("bytes_transferred", "Bytes transferred").unwrap(),
            processing_time: Histogram::new("processing_time", "Processing time").unwrap(),
            errors: Counter::new("errors", "Errors").unwrap(),
        }
    }

    pub fn record_packet(&self, size: u64, duration: Duration) {
        self.packets_processed.inc();
        self.bytes_transferred.inc_by(size as f64);
        self.processing_time.observe(duration.as_secs_f64());
    }
}
```

## Configuration

```toml
# config/server.toml

[server]
bind_address = "0.0.0.0"
port = 443
protocol = "udp"  # or "tcp" or "both"
max_connections = 10000
worker_threads = 0  # 0 = auto (num_cpus * 2)

[network]
tun_name = "hfp0"
tun_address = "10.8.0.1/24"
mtu = 1400
enable_ipv6 = true

[crypto]
mode = "maximum_security"  # or "balanced" or "performance"
ecc_enabled = true
hse_enabled = true
qrl_enabled = true
key_rotation_interval = 600  # seconds
key_rotation_bytes = 5000000  # bytes

[obfuscation]
enabled = true
stealth_mode = "maximum"
disguise_as = "nginx"
fallback_site = "https://example.com"

[performance]
zero_copy = true
io_uring = true
buffer_pool_size = 10000
buffer_size = 65536

[monitoring]
enable_metrics = true
metrics_port = 9090
log_level = "info"

[limits]
rate_limit_per_user = 100000000  # bytes/sec (100 MB/s)
max_streams_per_connection = 256
connection_timeout = 300  # seconds
```

## Deployment

### System Requirements

```
Minimum:
- CPU: 2 cores
- RAM: 2 GB
- Disk: 10 GB
- Network: 100 Mbps

Recommended:
- CPU: 4+ cores
- RAM: 4+ GB
- Disk: 20+ GB SSD
- Network: 1+ Gbps

OS: Ubuntu 20.04+, Debian 11+, CentOS 8+
```

### Installation

```bash
# One-line installation
curl -sSL https://install.lostlove.io | sudo bash -s -- \
  --port 443 \
  --stealth-mode maximum \
  --max-users 100
```

### Management Commands

```bash
# Start server
sudo systemctl start lostlove-server

# Stop server
sudo systemctl stop lostlove-server

# Restart server
sudo systemctl restart lostlove-server

# View logs
sudo journalctl -u lostlove-server -f

# Add user
sudo lostlove-admin add-user username

# Remove user
sudo lostlove-admin remove-user username

# Statistics
sudo lostlove-admin stats
```

## Security Considerations

1. **Firewall**: Открыть только необходимые порты
2. **SELinux/AppArmor**: Использовать профили безопасности
3. **User isolation**: Запуск под отдельным пользователем
4. **Rate limiting**: Защита от DDoS
5. **Fail2ban**: Автоматическая блокировка атакующих IP
6. **Regular updates**: Своевременное обновление
7. **Monitoring**: Постоянный мониторинг активности

## Scaling

### Horizontal Scaling

```
┌─────────────┐
│  HAProxy    │
└──────┬──────┘
       │
   ┌───┴───┬───────┬───────┐
   │       │       │       │
┌──▼──┐ ┌──▼──┐ ┌──▼──┐ ┌──▼──┐
│ LLP │ │ LLP │ │ LLP │ │ LLP │
│  #1 │ │  #2 │ │  #3 │ │  #4 │
└─────┘ └─────┘ └─────┘ └─────┘
```

### Database for User Management

```
┌──────────────┐
│  PostgreSQL  │  ← Central user database
└──────┬───────┘
       │
   ┌───┴───┬───────┬───────┐
   │       │       │       │
┌──▼──┐ ┌──▼──┐ ┌──▼──┐ ┌──▼──┐
│ LLP │ │ LLP │ │ LLP │ │ LLP │
└─────┘ └─────┘ └─────┘ └─────┘
```
