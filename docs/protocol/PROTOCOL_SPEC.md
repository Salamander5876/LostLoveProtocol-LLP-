# LostLove Protocol - Техническая спецификация

## Версия протокола: 1.0

## 1. Обзор

LostLove Protocol (LLP) - это протокол туннелирования с встроенным шифрованием и обфускацией, разработанный для максимальной безопасности и производительности.

## 2. Структура пакета

### 2.1 Базовый формат пакета

```
+--------------------+
|   Header (24 B)    |
+--------------------+
| Encrypted Payload  |
|     (Variable)     |
+--------------------+
```

### 2.2 Заголовок пакета (24 байта)

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Protocol ID (0xHF01)  |  Packet Type  |  Stream ID    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                     Sequence Number (8 bytes)                 |
|                                                                 |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Timestamp (8 bytes)                      |
|                                                                 |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Flags     |            Checksum (2 bytes)   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

#### Поля заголовка:

- **Protocol ID** (2 байта): Идентификатор протокола `0xHF01`
- **Packet Type** (1 байт): Тип пакета
  - `0x01` - DATA
  - `0x02` - ACK
  - `0x03` - HANDSHAKE_INIT
  - `0x04` - HANDSHAKE_RESPONSE
  - `0x05` - KEEPALIVE
  - `0x06` - DISCONNECT
- **Stream ID** (2 байта): Идентификатор потока (0-255)
- **Sequence Number** (8 байт): Порядковый номер пакета
- **Timestamp** (8 байт): Unix timestamp в миллисекундах
- **Flags** (1 байт): Битовые флаги
  - Bit 0: FIN (последний пакет в потоке)
  - Bit 1: RST (сброс соединения)
  - Bit 2: PRIORITY (высокий приоритет)
  - Bit 3-7: Зарезервировано
- **Checksum** (2 байта): CRC16 заголовка

### 2.3 Зашифрованная полезная нагрузка

```
+--------------------+
| Inner Header (16B) |
+--------------------+
|   Nonce (12 B)     |
+--------------------+
|   Actual Data      |
+--------------------+
|   Auth Tag (16 B)  |
+--------------------+
|   Padding          |
+--------------------+
```

#### Внутренний заголовок:

- **Real Length** (4 байта): Реальный размер данных
- **Compression Type** (1 байт): Тип сжатия
  - `0x00` - None
  - `0x01` - LZ4
  - `0x02` - Zstandard
- **Priority** (1 байт): Приоритет (0-255)
- **Reserved** (2 байта): Зарезервировано
- **Nonce** (12 байт): Уникальный nonce для шифрования

## 3. Протокол установления соединения

### 3.1 ZeroKnowledge Handshake

```
Client                                  Server
  |                                        |
  |------ INIT_HELLO ----------------------->|
  |   - Client_Random (64 bytes)            |
  |   - Supported_Curves[]                  |
  |   - Protocol_Version                    |
  |                                        |
  |<----- SERVER_CHALLENGE ------------------|
  |   - Server_Random (64 bytes)            |
  |   - Selected_Curve                      |
  |   - Puzzle_Challenge (PoW)              |
  |                                        |
  |------ CLIENT_PROOF --------------------->|
  |   - Puzzle_Solution                     |
  |   - ECDH_PublicKey                      |
  |   - Signed_Timestamp                    |
  |                                        |
  |<----- SESSION_ESTABLISHED ---------------|
  |   - ECDH_PublicKey                      |
  |   - Session_Token                       |
  |   - Initial_Curve_Parameters            |
  |                                        |
  |<====== Encrypted Data Flow ============>|
```

### 3.2 Генерация сессионных ключей

```
Master_Secret = HKDF-SHA512(
    ECDH_Shared_Secret,
    Client_Random || Server_Random,
    "LLP-v1-master"
)

Encryption_Key = HKDF-SHA512(Master_Secret, "encryption", 64)
MAC_Key = HKDF-SHA512(Master_Secret, "authentication", 64)
IV_Key = HKDF-SHA512(Master_Secret, "iv-generation", 32)
```

## 4. Мультиплексирование

### 4.1 Потоки (Streams)

- Каждое соединение поддерживает до 256 параллельных потоков
- Stream ID `0` зарезервирован для управляющих сообщений
- Streams ID `1-255` для пользовательских данных

### 4.2 Управление потоком

```
Flow Control Window = 256 KB (initial)
Max Window Size = 16 MB
Window Update Message:
  - Stream ID
  - Window Increment (4 bytes)
```

## 5. Обработка ошибок

### 5.1 Коды ошибок

- `0x01` - PROTOCOL_ERROR: Нарушение протокола
- `0x02` - INTERNAL_ERROR: Внутренняя ошибка сервера
- `0x03` - FLOW_CONTROL_ERROR: Ошибка управления потоком
- `0x04` - TIMEOUT: Таймаут
- `0x05` - STREAM_CLOSED: Поток закрыт
- `0x06` - CRYPTO_ERROR: Ошибка шифрования
- `0x07` - AUTHENTICATION_FAILED: Ошибка аутентификации

### 5.2 Обработка потери пакетов

- Используется выборочное подтверждение (SACK)
- Быстрая ретрансмиссия после 3 дублированных ACK
- Адаптивный таймаут ретрансмиссии (RTT-based)

## 6. Оптимизации производительности

### 6.1 Zero-Copy передача

- Использование sendfile() на Linux
- Memory-mapped I/O для больших передач
- Scatter-gather I/O

### 6.2 Адаптивное сжатие

```python
if packet_size > 512 and entropy(data) < 0.8:
    use_compression()
else:
    send_uncompressed()
```

### 6.3 Приоритезация трафика

```
Priority Levels:
- 255: Real-time (VoIP, Gaming)
- 192: Interactive (SSH, RDP)
- 128: Default (HTTP, General)
- 64: Bulk (Downloads)
- 0: Background
```

## 7. Характеристики безопасности

### 7.1 Ротация ключей

- Автоматическая ротация каждые 5 МБ данных
- Принудительная ротация каждые 60 минут
- Perfect Forward Secrecy

### 7.2 Защита от replay атак

- Используется строгая проверка Sequence Number
- Окно приема: 64 пакета
- Timestamp validation (±30 секунд)

### 7.3 Защита от анализа трафика

- Константный размер пакетов (padding)
- Timing jitter: случайные задержки 0-50мс
- Fake traffic generation во время idle

## 8. MTU и фрагментация

### 8.1 Обнаружение MTU

```
Initial MTU = 1280 bytes (IPv6 minimum)
Path MTU Discovery enabled
Adaptive MTU adjustment
```

### 8.2 Стратегия фрагментации

```
if packet_size > MTU:
    fragments = split_packet(packet, MTU - HEADER_SIZE)
    for fragment in fragments:
        send_with_fragment_header(fragment)
```

## 9. Версионирование

### 9.1 Текущая версия: 1.0

```
Protocol Version Field: 0x01
Backward compatibility: None (initial version)
```

### 9.2 Обновления протокола

- Minor updates: совместимы внутри мажорной версии
- Major updates: могут ломать совместимость
- Negotiation during handshake

## 10. Ограничения

- Максимальный размер пакета: 64 KB
- Максимальное количество потоков: 256
- Максимальное время сессии: 24 часа (ре-рукопожатие)
- Максимальная скорость на поток: нет ограничений
