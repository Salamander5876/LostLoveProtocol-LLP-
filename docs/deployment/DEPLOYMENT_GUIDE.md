# LostLove Protocol - Руководство по развертыванию

## Содержание

1. [Требования к серверу](#требования-к-серверу)
2. [Быстрая установка](#быстрая-установка)
3. [Ручная установка](#ручная-установка)
4. [Конфигурация](#конфигурация)
5. [Управление пользователями](#управление-пользователями)
6. [Мониторинг](#мониторинг)
7. [Обновление](#обновление)
8. [Troubleshooting](#troubleshooting)

## Требования к серверу

### Минимальные требования

```
CPU:      2 cores
RAM:      2 GB
Disk:     10 GB
Network:  100 Mbps
OS:       Ubuntu 20.04+, Debian 11+, CentOS 8+
```

### Рекомендуемые требования

```
CPU:      4+ cores (8+ для 1000+ пользователей)
RAM:      4+ GB (8+ для 1000+ пользователей)
Disk:     20+ GB SSD
Network:  1+ Gbps
OS:       Ubuntu 22.04 LTS
```

### Провайдеры VPS

Рекомендуемые провайдеры:

```
1. DigitalOcean
   - Datacenter: выбирайте близкий к пользователям
   - Plan: Basic Droplet $12/month (2 CPU, 2 GB RAM)

2. Vultr
   - High Frequency
   - От $12/month

3. Linode
   - Shared CPU
   - От $12/month

4. Hetzner
   - Cloud Server
   - От €5/month (дешевле, но может быть заблокирован в некоторых странах)
```

## Быстрая установка

### Автоматическая установка (рекомендуется)

```bash
# Скачайте и запустите скрипт установки
curl -sSL https://install.lostlove.io | sudo bash

# Или с параметрами
curl -sSL https://install.lostlove.io | sudo bash -s -- \
  --port 443 \
  --stealth-mode maximum \
  --max-users 100 \
  --admin-email your@email.com
```

### Параметры установки

```bash
--port PORT               # Порт сервера (по умолчанию: 443)
--protocol PROTOCOL       # tcp, udp или both (по умолчанию: both)
--stealth-mode MODE       # normal, high, maximum (по умолчанию: maximum)
--disguise-as SERVICE     # nginx, apache, cloudflare (по умолчанию: nginx)
--max-users COUNT         # Максимальное количество пользователей
--bandwidth LIMIT         # Ограничение bandwidth (unlimited по умолчанию)
--admin-email EMAIL       # Email администратора
--domain DOMAIN           # Доменное имя (опционально)
--enable-tls-cert         # Получить Let's Encrypt сертификат
--no-firewall             # Не настраивать firewall
--unattended              # Неинтерактивная установка
```

### Пример полной установки

```bash
curl -sSL https://install.lostlove.io | sudo bash -s -- \
  --port 443 \
  --protocol both \
  --stealth-mode maximum \
  --disguise-as nginx \
  --max-users 500 \
  --admin-email admin@example.com \
  --domain vpn.example.com \
  --enable-tls-cert \
  --unattended
```

## Ручная установка

### 1. Установка зависимостей

#### Ubuntu/Debian

```bash
# Обновляем систему
sudo apt update && sudo apt upgrade -y

# Устанавливаем зависимости
sudo apt install -y \
  build-essential \
  cmake \
  pkg-config \
  libssl-dev \
  libsqlite3-dev \
  iptables \
  net-tools \
  curl \
  git

# Устанавливаем Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### CentOS/RHEL

```bash
# Обновляем систему
sudo yum update -y

# Устанавливаем зависимости
sudo yum install -y \
  gcc \
  gcc-c++ \
  cmake \
  openssl-devel \
  sqlite-devel \
  iptables \
  net-tools \
  curl \
  git

# Устанавливаем Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Сборка LostLove Server

```bash
# Клонируем репозиторий
git clone https://github.com/yourusername/lostlove-protocol.git
cd lostlove-protocol/server

# Собираем release версию
cargo build --release

# Копируем бинарник
sudo cp target/release/lostlove-server /usr/local/bin/

# Создаем директории
sudo mkdir -p /etc/lostlove
sudo mkdir -p /var/log/lostlove
sudo mkdir -p /var/lib/lostlove
```

### 3. Создание конфигурации

```bash
# Создаем конфигурационный файл
sudo cat > /etc/lostlove/server.toml << EOF
[server]
bind_address = "0.0.0.0"
port = 443
protocol = "both"
max_connections = 10000
worker_threads = 0

[network]
tun_name = "hfp0"
tun_address = "10.8.0.1/24"
mtu = 1400
enable_ipv6 = true

[crypto]
mode = "maximum_security"
ecc_enabled = true
hse_enabled = true
qrl_enabled = true

[obfuscation]
enabled = true
stealth_mode = "maximum"
disguise_as = "nginx"
fallback_site = "https://example.com"

[monitoring]
enable_metrics = true
metrics_port = 9090
log_level = "info"

[limits]
rate_limit_per_user = 100000000
max_streams_per_connection = 256
connection_timeout = 300
EOF
```

### 4. Создание systemd сервиса

```bash
# Создаем systemd unit
sudo cat > /etc/systemd/system/lostlove-server.service << EOF
[Unit]
Description=LostLove VPN Server
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/lostlove-server --config /etc/lostlove/server.toml
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/lostlove /var/lib/lostlove

[Install]
WantedBy=multi-user.target
EOF

# Перезагружаем systemd
sudo systemctl daemon-reload

# Включаем автозапуск
sudo systemctl enable lostlove-server
```

### 5. Настройка firewall

```bash
# UFW (Ubuntu/Debian)
sudo ufw allow 443/tcp
sudo ufw allow 443/udp
sudo ufw allow 22/tcp
sudo ufw enable

# firewalld (CentOS/RHEL)
sudo firewall-cmd --permanent --add-port=443/tcp
sudo firewall-cmd --permanent --add-port=443/udp
sudo firewall-cmd --permanent --add-port=22/tcp
sudo firewall-cmd --reload

# iptables (если используется напрямую)
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
sudo iptables -A INPUT -p udp --dport 443 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 22 -j ACCEPT
sudo iptables-save | sudo tee /etc/iptables/rules.v4
```

### 6. Настройка IP forwarding и NAT

```bash
# Включаем IP forwarding
echo "net.ipv4.ip_forward = 1" | sudo tee -a /etc/sysctl.conf
echo "net.ipv6.conf.all.forwarding = 1" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# Настраиваем NAT
export DEFAULT_IF=$(ip route | grep default | awk '{print $5}')

sudo iptables -t nat -A POSTROUTING -s 10.8.0.0/24 -o $DEFAULT_IF -j MASQUERADE
sudo iptables -A FORWARD -i hfp0 -o $DEFAULT_IF -j ACCEPT
sudo iptables -A FORWARD -i $DEFAULT_IF -o hfp0 -m state --state RELATED,ESTABLISHED -j ACCEPT

# Сохраняем правила
sudo iptables-save | sudo tee /etc/iptables/rules.v4

# Для IPv6
sudo ip6tables -t nat -A POSTROUTING -s fd00:8::/64 -o $DEFAULT_IF -j MASQUERADE
sudo ip6tables -A FORWARD -i hfp0 -o $DEFAULT_IF -j ACCEPT
sudo ip6tables -A FORWARD -i $DEFAULT_IF -o hfp0 -m state --state RELATED,ESTABLISHED -j ACCEPT
sudo ip6tables-save | sudo tee /etc/iptables/rules.v6
```

### 7. Запуск сервера

```bash
# Запускаем сервер
sudo systemctl start lostlove-server

# Проверяем статус
sudo systemctl status lostlove-server

# Смотрим логи
sudo journalctl -u lostlove-server -f
```

## Конфигурация

### Основные параметры

#### Изменение порта

```toml
[server]
port = 8443  # Изменить на нужный порт
```

```bash
# Не забудьте открыть порт в firewall
sudo ufw allow 8443/tcp
sudo ufw allow 8443/udp

# Перезапустите сервер
sudo systemctl restart lostlove-server
```

#### Настройка обфускации

```toml
[obfuscation]
enabled = true
stealth_mode = "maximum"  # normal, high, maximum

# Выбор типа маскировки
disguise_as = "nginx"  # nginx, apache, cloudflare

# Fallback сайт (показывается при обычном HTTP запросе)
fallback_site = "https://example.com"

[obfuscation.traffic_shaping]
fake_traffic_ratio = 0.15
padding_enabled = true
timing_jitter = true

[obfuscation.dpi_evasion]
enabled = true
sensitivity = "high"  # low, medium, high, paranoid
active_defense = true
```

#### Настройка криптографии

```toml
[crypto]
mode = "balanced"  # performance, balanced, maximum_security

# Отдельные слои
ecc_enabled = true
hse_enabled = true
qrl_enabled = false  # Можно отключить для производительности

# Ротация ключей
key_rotation_interval = 600  # секунды
key_rotation_bytes = 5000000  # байты
```

#### Производительность

```toml
[performance]
zero_copy = true  # Использовать zero-copy I/O
io_uring = true   # Использовать io_uring (Linux 5.1+)

buffer_pool_size = 10000  # Размер пула буферов
buffer_size = 65536       # Размер буфера

[server]
worker_threads = 8  # Количество рабочих потоков (0 = auto)

[limits]
rate_limit_per_user = 100000000  # 100 MB/s на пользователя
max_streams_per_connection = 256
connection_timeout = 300
```

## Управление пользователями

### Создание первого пользователя

```bash
# При первом запуске создается admin пользователь
sudo lostlove-admin first-user

# Или создать вручную
sudo lostlove-admin add-user admin --admin --password "your-secure-password"
```

### Добавление пользователей

```bash
# Интерактивно
sudo lostlove-admin add-user username

# С параметрами
sudo lostlove-admin add-user john \
  --password "secure-password" \
  --email "john@example.com" \
  --bandwidth-limit 50000000 \
  --expire-date "2024-12-31"

# Генерация случайного пароля
sudo lostlove-admin add-user jane --generate-password
```

### Генерация конфигурации клиента

```bash
# QR код для быстрого подключения
sudo lostlove-admin export-config username --qr

# Конфигурационный файл
sudo lostlove-admin export-config username --output /tmp/config.json

# URL для подключения
sudo lostlove-admin export-config username --url
```

### Управление пользователями

```bash
# Список пользователей
sudo lostlove-admin list-users

# Информация о пользователе
sudo lostlove-admin user-info username

# Изменение пароля
sudo lostlove-admin change-password username

# Деактивация пользователя
sudo lostlove-admin disable-user username

# Активация пользователя
sudo lostlove-admin enable-user username

# Удаление пользователя
sudo lostlove-admin remove-user username

# Статистика пользователя
sudo lostlove-admin user-stats username
```

## Мониторинг

### Логи

```bash
# Просмотр логов в реальном времени
sudo journalctl -u lostlove-server -f

# Последние 100 строк
sudo journalctl -u lostlove-server -n 100

# Логи за сегодня
sudo journalctl -u lostlove-server --since today

# Логи с фильтром
sudo journalctl -u lostlove-server | grep ERROR
```

### Метрики (Prometheus)

```bash
# Метрики доступны на порту 9090
curl http://localhost:9090/metrics

# Основные метрики:
# - active_connections
# - packets_processed_total
# - bytes_transferred_total
# - processing_time_seconds
# - errors_total
```

### Статистика сервера

```bash
# Общая статистика
sudo lostlove-admin stats

# Активные соединения
sudo lostlove-admin active-connections

# Использование bandwidth
sudo lostlove-admin bandwidth

# Топ пользователей по трафику
sudo lostlove-admin top-users
```

### Мониторинг с Grafana

```bash
# Установка Prometheus
sudo apt install -y prometheus

# Конфигурация Prometheus
sudo cat > /etc/prometheus/prometheus.yml << EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'lostlove'
    static_configs:
      - targets: ['localhost:9090']
EOF

sudo systemctl restart prometheus

# Установка Grafana
sudo apt install -y grafana

sudo systemctl enable grafana-server
sudo systemctl start grafana-server

# Grafana будет доступна на http://your-server:3000
# Логин: admin, пароль: admin (измените при первом входе)
```

## Обновление

### Автоматическое обновление

```bash
# Обновление до последней версии
curl -sSL https://install.lostlove.io/update | sudo bash

# Или
sudo lostlove-admin update
```

### Ручное обновление

```bash
# Остановить сервер
sudo systemctl stop lostlove-server

# Создать backup
sudo lostlove-admin backup /tmp/lostlove-backup-$(date +%Y%m%d).tar.gz

# Скачать новую версию
cd /tmp
wget https://github.com/yourusername/lostlove-protocol/releases/latest/download/lostlove-server-linux-x64.tar.gz

# Распаковать
tar xzf lostlove-server-linux-x64.tar.gz

# Заменить бинарник
sudo cp lostlove-server /usr/local/bin/

# Запустить сервер
sudo systemctl start lostlove-server

# Проверить статус
sudo systemctl status lostlove-server
```

## Backup и Restore

### Создание backup

```bash
# Полный backup (конфигурация + база данных)
sudo lostlove-admin backup /backup/lostlove-$(date +%Y%m%d).tar.gz

# Только база данных
sudo cp /var/lib/lostlove/users.db /backup/users-$(date +%Y%m%d).db

# Только конфигурация
sudo cp /etc/lostlove/server.toml /backup/server-$(date +%Y%m%d).toml
```

### Восстановление

```bash
# Остановить сервер
sudo systemctl stop lostlove-server

# Восстановить из backup
sudo lostlove-admin restore /backup/lostlove-20240101.tar.gz

# Или вручную
sudo tar xzf /backup/lostlove-20240101.tar.gz -C /

# Запустить сервер
sudo systemctl start lostlove-server
```

### Автоматический backup

```bash
# Создать скрипт backup
sudo cat > /usr/local/bin/lostlove-backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/backup/lostlove"
DATE=$(date +%Y%m%d)

mkdir -p $BACKUP_DIR

lostlove-admin backup $BACKUP_DIR/lostlove-$DATE.tar.gz

# Удалить старые backup (старше 30 дней)
find $BACKUP_DIR -name "lostlove-*.tar.gz" -mtime +30 -delete
EOF

sudo chmod +x /usr/local/bin/lostlove-backup.sh

# Добавить в cron (ежедневно в 3:00)
echo "0 3 * * * /usr/local/bin/lostlove-backup.sh" | sudo crontab -
```

## Troubleshooting

### Сервер не запускается

```bash
# Проверить логи
sudo journalctl -u lostlove-server -n 100

# Проверить конфигурацию
sudo lostlove-server --config /etc/lostlove/server.toml --check-config

# Проверить порты
sudo netstat -tulpn | grep 443

# Проверить разрешения
sudo ls -la /etc/lostlove
sudo ls -la /var/lib/lostlove
```

### Клиент не может подключиться

```bash
# Проверить firewall
sudo ufw status
sudo iptables -L -n

# Проверить, слушает ли сервер
sudo ss -tulpn | grep lostlove

# Тест подключения с клиента
telnet your-server-ip 443

# Проверить логи на сервере
sudo journalctl -u lostlove-server -f
```

### Низкая скорость

```bash
# Проверить CPU usage
htop

# Проверить настройки производительности
grep -E "(zero_copy|io_uring|worker_threads)" /etc/lostlove/server.toml

# Проверить MTU
ip link show hfp0

# Оптимизировать TCP
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728
sudo sysctl -w net.ipv4.tcp_rmem="4096 87380 67108864"
sudo sysctl -w net.ipv4.tcp_wmem="4096 65536 67108864"
```

### Высокое использование памяти

```bash
# Проверить память
free -h

# Уменьшить buffer pool
# В /etc/lostlove/server.toml:
[performance]
buffer_pool_size = 5000  # уменьшить

# Ограничить количество соединений
[server]
max_connections = 1000  # уменьшить

sudo systemctl restart lostlove-server
```

### Сервер заблокирован провайдером

```bash
# Включить максимальную обфускацию
# В /etc/lostlove/server.toml:
[obfuscation]
stealth_mode = "maximum"
disguise_as = "nginx"

[obfuscation.dpi_evasion]
enabled = true
sensitivity = "paranoid"
active_defense = true

sudo systemctl restart lostlove-server

# Изменить порт на 443 (HTTPS)
[server]
port = 443

# Получить Let's Encrypt сертификат
sudo lostlove-admin setup-tls --domain your-domain.com
```

## Безопасность

### Рекомендации

```bash
# 1. Регулярно обновляйте систему
sudo apt update && sudo apt upgrade -y

# 2. Настройте fail2ban
sudo apt install -y fail2ban

sudo cat > /etc/fail2ban/jail.local << EOF
[lostlove]
enabled = true
port = 443
filter = lostlove
logpath = /var/log/lostlove/server.log
maxretry = 5
bantime = 3600
EOF

sudo systemctl restart fail2ban

# 3. Ограничьте SSH доступ
sudo ufw allow from YOUR_IP to any port 22

# 4. Используйте SSH ключи
ssh-keygen -t ed25519
# Скопируйте публичный ключ на сервер

# 5. Отключите root login
sudo sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
sudo systemctl restart sshd

# 6. Мониторинг
sudo apt install -y logwatch
```

### Audit

```bash
# Проверить открытые порты
sudo ss -tulpn

# Проверить запущенные процессы
ps aux | grep lostlove

# Проверить пользователей
sudo lostlove-admin list-users

# Проверить логи аутентификации
sudo journalctl -u lostlove-server | grep "authentication"

# Проверить активные соединения
sudo lostlove-admin active-connections
```

## Производственное развертывание

### High Availability (HA)

```
┌─────────────┐
│   HAProxy   │ (Load Balancer)
└──────┬──────┘
       │
   ┌───┴───┬───────┬───────┐
   │       │       │       │
┌──▼──┐ ┌──▼──┐ ┌──▼──┐ ┌──▼──┐
│ LLP │ │ LLP │ │ LLP │ │ LLP │
│  #1 │ │  #2 │ │  #3 │ │  #4 │
└─────┘ └─────┘ └─────┘ └─────┘
       │
┌──────▼───────┐
│  PostgreSQL  │ (Shared DB)
└──────────────┘
```

### Multi-Region Deployment

```
Region: US-East          Region: EU-West        Region: Asia-Pacific
┌─────────────┐          ┌─────────────┐        ┌─────────────┐
│ LLP Cluster │          │ LLP Cluster │        │ LLP Cluster │
└──────┬──────┘          └──────┬──────┘        └──────┬──────┘
       │                        │                      │
       └────────────┬───────────┴──────────────────────┘
                    │
            ┌───────▼────────┐
            │  GeoDNS/Route  │
            │  DNS Failover  │
            └────────────────┘
```

## Заключение

После успешной установки:

1. Сервер работает на порту 443 (или выбранном)
2. Пользователи могут подключаться через клиент
3. Трафик полностью зашифрован и замаскирован
4. Мониторинг и логи настроены
5. Автоматические backup работают

Для вопросов и поддержки:
- GitHub Issues: https://github.com/yourusername/lostlove-protocol/issues
- Documentation: https://docs.lostlove.io
- Community: https://community.lostlove.io
