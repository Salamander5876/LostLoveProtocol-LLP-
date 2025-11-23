# LostLove Client (Windows) - Архитектура

## Обзор

LostLove Client - это клиентское приложение для Windows, обеспечивающее подключение к LostLove VPN серверу с графическим интерфейсом и встроенной обфускацией.

## Архитектура системы

```
┌────────────────────────────────────────────────────────┐
│                 Presentation Layer                     │
│  ┌──────────────────────────────────────────────────┐ │
│  │       GUI (Electron + React)                     │ │
│  │  ┌─────────────────────────────────────────────┐ │ │
│  │  │  - Dashboard                                 │ │ │
│  │  │  - Server Selection                          │ │ │
│  │  │  - Statistics                                │ │ │
│  │  │  - Settings                                  │ │ │
│  │  │  - Logs                                      │ │ │
│  │  └─────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
                          │
                          ▼ IPC (Named Pipes)
┌────────────────────────────────────────────────────────┐
│                  Service Layer (C++)                   │
│  ┌──────────────────────────────────────────────────┐ │
│  │         Windows Service (Background)             │ │
│  │  ┌─────────────────────────────────────────────┐ │ │
│  │  │  - Connection Manager                       │ │ │
│  │  │  - Protocol Implementation (LLP)            │ │ │
│  │  │  - Crypto Engine (QuantumShield)            │ │ │
│  │  │  - Obfuscation Engine (Chameleon)           │ │ │
│  │  │  - Auto-reconnect Logic                     │ │ │
│  │  └─────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────┐
│               Network Layer (Kernel)                   │
│  ┌──────────────────────────────────────────────────┐ │
│  │     TUN/TAP Driver (WinTUN)                      │ │
│  │  ┌─────────────────────────────────────────────┐ │ │
│  │  │  - Virtual Network Adapter                  │ │ │
│  │  │  - Packet Capture & Injection               │ │ │
│  │  │  - Routing Table Modification               │ │ │
│  │  └─────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
                          │
                          ▼
                   Internet (VPN Server)
```

## Структура проекта

```
client/
├── desktop-windows/
│   ├── gui/                          # Electron GUI
│   │   ├── src/
│   │   │   ├── main/                 # Main process
│   │   │   │   ├── index.ts
│   │   │   │   ├── ipc-handlers.ts
│   │   │   │   └── service-manager.ts
│   │   │   │
│   │   │   ├── renderer/             # Renderer process
│   │   │   │   ├── App.tsx
│   │   │   │   ├── pages/
│   │   │   │   │   ├── Dashboard.tsx
│   │   │   │   │   ├── Servers.tsx
│   │   │   │   │   ├── Statistics.tsx
│   │   │   │   │   └── Settings.tsx
│   │   │   │   ├── components/
│   │   │   │   │   ├── ConnectionButton.tsx
│   │   │   │   │   ├── ServerCard.tsx
│   │   │   │   │   ├── StatsChart.tsx
│   │   │   │   │   └── LogViewer.tsx
│   │   │   │   └── hooks/
│   │   │   │       ├── useConnection.ts
│   │   │   │       └── useStatistics.ts
│   │   │   │
│   │   │   └── shared/
│   │   │       ├── types.ts
│   │   │       └── constants.ts
│   │   │
│   │   ├── package.json
│   │   ├── webpack.config.js
│   │   └── tsconfig.json
│   │
│   ├── service/                      # C++ Windows Service
│   │   ├── src/
│   │   │   ├── main.cpp
│   │   │   ├── service.h/cpp         # Windows Service
│   │   │   ├── connection/
│   │   │   │   ├── connection_manager.h/cpp
│   │   │   │   ├── session.h/cpp
│   │   │   │   └── auto_reconnect.h/cpp
│   │   │   │
│   │   │   ├── protocol/             # LLP Implementation
│   │   │   │   ├── packet.h/cpp
│   │   │   │   ├── handshake.h/cpp
│   │   │   │   ├── stream.h/cpp
│   │   │   │   └── codec.h/cpp
│   │   │   │
│   │   │   ├── crypto/               # Cryptography
│   │   │   │   ├── quantum_shield.h/cpp
│   │   │   │   ├── ecc.h/cpp
│   │   │   │   ├── hse.h/cpp
│   │   │   │   ├── qrl.h/cpp
│   │   │   │   └── key_manager.h/cpp
│   │   │   │
│   │   │   ├── obfuscation/          # Traffic Obfuscation
│   │   │   │   ├── chameleon.h/cpp
│   │   │   │   ├── traffic_shaper.h/cpp
│   │   │   │   ├── dpi_evasion.h/cpp
│   │   │   │   └── domain_fronting.h/cpp
│   │   │   │
│   │   │   ├── network/              # Network Layer
│   │   │   │   ├── tun_adapter.h/cpp
│   │   │   │   ├── router.h/cpp
│   │   │   │   ├── dns.h/cpp
│   │   │   │   └── firewall.h/cpp
│   │   │   │
│   │   │   ├── ipc/                  # IPC with GUI
│   │   │   │   ├── named_pipe_server.h/cpp
│   │   │   │   └── message_handler.h/cpp
│   │   │   │
│   │   │   └── utils/
│   │   │       ├── config.h/cpp
│   │   │       ├── logger.h/cpp
│   │   │       └── registry.h/cpp
│   │   │
│   │   ├── CMakeLists.txt
│   │   └── vcpkg.json
│   │
│   ├── installer/                    # NSIS Installer
│   │   ├── installer.nsi
│   │   ├── install.ico
│   │   └── uninstall.ico
│   │
│   └── resources/
│       ├── icon.ico
│       ├── tray-icon.ico
│       └── app.manifest
│
└── core/                             # Shared Core Library (C++)
    ├── include/
    │   └── lostlove/
    │       ├── protocol.h
    │       ├── crypto.h
    │       └── obfuscation.h
    └── src/
        └── ...
```

## GUI Layer (Electron + React)

### 1. Main Process

```typescript
// gui/src/main/index.ts

import { app, BrowserWindow, ipcMain } from 'electron';
import { ServiceManager } from './service-manager';

class LostLoveApp {
  private mainWindow: BrowserWindow | null = null;
  private serviceManager: ServiceManager;

  constructor() {
    this.serviceManager = new ServiceManager();
    this.setupIPC();
  }

  async init() {
    await app.whenReady();

    // Start Windows Service
    await this.serviceManager.start();

    // Create main window
    this.createMainWindow();

    // Setup tray icon
    this.createTrayIcon();
  }

  private createMainWindow() {
    this.mainWindow = new BrowserWindow({
      width: 1000,
      height: 700,
      minWidth: 800,
      minHeight: 600,
      frame: true,
      backgroundColor: '#1a1a1a',
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
        preload: path.join(__dirname, 'preload.js')
      }
    });

    this.mainWindow.loadFile('index.html');
  }

  private setupIPC() {
    // Connect to server
    ipcMain.handle('connect', async (event, serverConfig) => {
      return await this.serviceManager.connect(serverConfig);
    });

    // Disconnect
    ipcMain.handle('disconnect', async () => {
      return await this.serviceManager.disconnect();
    });

    // Get connection status
    ipcMain.handle('get-status', async () => {
      return await this.serviceManager.getStatus();
    });

    // Get statistics
    ipcMain.handle('get-stats', async () => {
      return await this.serviceManager.getStatistics();
    });

    // Get server list
    ipcMain.handle('get-servers', async () => {
      return await this.serviceManager.getServerList();
    });

    // Update settings
    ipcMain.handle('update-settings', async (event, settings) => {
      return await this.serviceManager.updateSettings(settings);
    });

    // Listen for status updates
    this.serviceManager.on('status-changed', (status) => {
      this.mainWindow?.webContents.send('status-update', status);
    });

    this.serviceManager.on('stats-update', (stats) => {
      this.mainWindow?.webContents.send('stats-update', stats);
    });
  }
}

const lostloveApp = new LostLoveApp();
lostloveApp.init();
```

### 2. Service Manager

```typescript
// gui/src/main/service-manager.ts

import { EventEmitter } from 'events';
import * as net from 'net';

export class ServiceManager extends EventEmitter {
  private pipe: net.Socket | null = null;
  private readonly pipeName = '\\\\.\\pipe\\LostLoveService';

  async start(): Promise<void> {
    // Check if service is running
    const isRunning = await this.checkServiceStatus();

    if (!isRunning) {
      // Start Windows Service
      await this.startWindowsService();
    }

    // Connect to service via named pipe
    await this.connectToPipe();

    // Start listening for updates
    this.startStatusPolling();
  }

  private async connectToPipe(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.pipe = net.connect(this.pipeName, () => {
        console.log('Connected to LostLove Service');
        resolve();
      });

      this.pipe.on('data', (data) => {
        this.handleServiceMessage(data);
      });

      this.pipe.on('error', (err) => {
        console.error('Pipe error:', err);
        reject(err);
      });
    });
  }

  async connect(serverConfig: ServerConfig): Promise<Result> {
    const message = {
      type: 'connect',
      payload: serverConfig
    };

    return await this.sendMessage(message);
  }

  async disconnect(): Promise<Result> {
    const message = {
      type: 'disconnect'
    };

    return await this.sendMessage(message);
  }

  async getStatus(): Promise<ConnectionStatus> {
    const message = {
      type: 'get-status'
    };

    const response = await this.sendMessage(message);
    return response.payload;
  }

  async getStatistics(): Promise<Statistics> {
    const message = {
      type: 'get-stats'
    };

    const response = await this.sendMessage(message);
    return response.payload;
  }

  private async sendMessage(message: any): Promise<any> {
    return new Promise((resolve, reject) => {
      if (!this.pipe) {
        reject(new Error('Not connected to service'));
        return;
      }

      const messageStr = JSON.stringify(message);
      const messageId = Date.now();

      // Send message
      this.pipe.write(`${messageId}:${messageStr}\n`);

      // Wait for response
      const responseHandler = (data: Buffer) => {
        const response = JSON.parse(data.toString());
        if (response.id === messageId) {
          this.pipe?.removeListener('data', responseHandler);
          resolve(response);
        }
      };

      this.pipe.on('data', responseHandler);

      // Timeout
      setTimeout(() => {
        this.pipe?.removeListener('data', responseHandler);
        reject(new Error('Request timeout'));
      }, 10000);
    });
  }

  private handleServiceMessage(data: Buffer) {
    try {
      const message = JSON.parse(data.toString());

      switch (message.type) {
        case 'status-changed':
          this.emit('status-changed', message.payload);
          break;

        case 'stats-update':
          this.emit('stats-update', message.payload);
          break;

        case 'error':
          this.emit('error', message.payload);
          break;
      }
    } catch (err) {
      console.error('Failed to parse service message:', err);
    }
  }
}
```

### 3. React Components

```typescript
// gui/src/renderer/pages/Dashboard.tsx

import React, { useEffect, useState } from 'react';
import { useConnection } from '../hooks/useConnection';
import { ConnectionButton } from '../components/ConnectionButton';
import { StatsChart } from '../components/StatsChart';

export const Dashboard: React.FC = () => {
  const { status, stats, connect, disconnect } = useConnection();
  const [selectedServer, setSelectedServer] = useState<Server | null>(null);

  return (
    <div className="dashboard">
      <div className="connection-panel">
        <h2>Connection Status</h2>

        <div className="status-indicator">
          <span className={`status-dot ${status.connected ? 'connected' : 'disconnected'}`} />
          <span className="status-text">
            {status.connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>

        {status.connected && (
          <div className="connection-info">
            <div className="info-row">
              <span>Server:</span>
              <span>{status.server}</span>
            </div>
            <div className="info-row">
              <span>IP Address:</span>
              <span>{status.ip}</span>
            </div>
            <div className="info-row">
              <span>Latency:</span>
              <span>{status.latency}ms</span>
            </div>
            <div className="info-row">
              <span>Obfuscation:</span>
              <span>{status.obfuscation_mode}</span>
            </div>
          </div>
        )}

        <ConnectionButton
          connected={status.connected}
          onClick={() => status.connected ? disconnect() : connect(selectedServer)}
        />
      </div>

      <div className="stats-panel">
        <h2>Statistics</h2>

        <div className="stats-grid">
          <div className="stat-card">
            <div className="stat-value">{formatBytes(stats.download_total)}</div>
            <div className="stat-label">Downloaded</div>
          </div>

          <div className="stat-card">
            <div className="stat-value">{formatBytes(stats.upload_total)}</div>
            <div className="stat-label">Uploaded</div>
          </div>

          <div className="stat-card">
            <div className="stat-value">{formatSpeed(stats.download_speed)}</div>
            <div className="stat-label">Download Speed</div>
          </div>

          <div className="stat-card">
            <div className="stat-value">{formatSpeed(stats.upload_speed)}</div>
            <div className="stat-label">Upload Speed</div>
          </div>
        </div>

        <StatsChart data={stats.history} />
      </div>
    </div>
  );
};
```

## Service Layer (C++)

### 1. Windows Service

```cpp
// service/src/service.cpp

#include "service.h"
#include "connection/connection_manager.h"
#include "ipc/named_pipe_server.h"

class LostLoveService : public WindowsService {
public:
    LostLoveService()
        : WindowsService(L"LostLoveService", L"LostLove VPN Service")
    {
        m_connection_manager = std::make_unique<ConnectionManager>();
        m_ipc_server = std::make_unique<NamedPipeServer>();
    }

    void OnStart(DWORD argc, LPWSTR* argv) override {
        LogInfo("LostLove Service starting...");

        // Initialize components
        m_connection_manager->Initialize();
        m_ipc_server->Start();

        // Setup IPC message handlers
        SetupIPCHandlers();

        // Start worker thread
        m_worker_thread = std::thread([this]() {
            WorkerThread();
        });

        LogInfo("LostLove Service started");
    }

    void OnStop() override {
        LogInfo("LostLove Service stopping...");

        m_running = false;

        // Disconnect if connected
        if (m_connection_manager->IsConnected()) {
            m_connection_manager->Disconnect();
        }

        // Stop IPC server
        m_ipc_server->Stop();

        // Wait for worker thread
        if (m_worker_thread.joinable()) {
            m_worker_thread.join();
        }

        LogInfo("LostLove Service stopped");
    }

private:
    void SetupIPCHandlers() {
        m_ipc_server->RegisterHandler("connect", [this](const json& msg) {
            return HandleConnect(msg);
        });

        m_ipc_server->RegisterHandler("disconnect", [this](const json& msg) {
            return HandleDisconnect(msg);
        });

        m_ipc_server->RegisterHandler("get-status", [this](const json& msg) {
            return HandleGetStatus(msg);
        });

        m_ipc_server->RegisterHandler("get-stats", [this](const json& msg) {
            return HandleGetStats(msg);
        });
    }

    json HandleConnect(const json& msg) {
        try {
            ServerConfig config = msg["payload"];

            bool success = m_connection_manager->Connect(config);

            return {
                {"success", success},
                {"message", success ? "Connected" : "Connection failed"}
            };
        }
        catch (const std::exception& e) {
            return {
                {"success", false},
                {"message", e.what()}
            };
        }
    }

    json HandleDisconnect(const json& msg) {
        m_connection_manager->Disconnect();

        return {
            {"success", true},
            {"message", "Disconnected"}
        };
    }

    json HandleGetStatus(const json& msg) {
        auto status = m_connection_manager->GetStatus();

        return {
            {"success", true},
            {"payload", status}
        };
    }

    json HandleGetStats(const json& msg) {
        auto stats = m_connection_manager->GetStatistics();

        return {
            {"success", true},
            {"payload", stats}
        };
    }

    void WorkerThread() {
        while (m_running) {
            // Process packets
            m_connection_manager->ProcessPackets();

            // Send status updates to GUI
            if (m_connection_manager->IsConnected()) {
                auto stats = m_connection_manager->GetStatistics();
                m_ipc_server->Broadcast("stats-update", stats);
            }

            std::this_thread::sleep_for(std::chrono::milliseconds(100));
        }
    }

private:
    std::unique_ptr<ConnectionManager> m_connection_manager;
    std::unique_ptr<NamedPipeServer> m_ipc_server;
    std::thread m_worker_thread;
    std::atomic<bool> m_running{ true };
};

int wmain(int argc, wchar_t* argv[]) {
    LostLoveService service;
    return service.Run();
}
```

### 2. Connection Manager

```cpp
// service/src/connection/connection_manager.h

class ConnectionManager {
public:
    ConnectionManager();
    ~ConnectionManager();

    bool Initialize();
    bool Connect(const ServerConfig& config);
    void Disconnect();

    bool IsConnected() const;
    ConnectionStatus GetStatus() const;
    Statistics GetStatistics() const;

    void ProcessPackets();

private:
    void HandshakeThread();
    void SendThread();
    void RecvThread();

    bool PerformHandshake();
    void SetupTunAdapter();
    void ConfigureRouting();

private:
    std::unique_ptr<TunAdapter> m_tun;
    std::unique_ptr<CryptoEngine> m_crypto;
    std::unique_ptr<ObfuscationEngine> m_obfuscation;

    std::unique_ptr<Session> m_session;
    ServerConfig m_server_config;

    std::atomic<bool> m_connected{ false };
    std::atomic<bool> m_running{ false };

    std::thread m_send_thread;
    std::thread m_recv_thread;

    // Lock-free queues
    moodycamel::ConcurrentQueue<Packet> m_send_queue;
    moodycamel::ConcurrentQueue<Packet> m_recv_queue;

    // Statistics
    std::atomic<uint64_t> m_bytes_sent{ 0 };
    std::atomic<uint64_t> m_bytes_received{ 0 };
    std::atomic<uint64_t> m_packets_sent{ 0 };
    std::atomic<uint64_t> m_packets_received{ 0 };
};
```

### 3. TUN Adapter

```cpp
// service/src/network/tun_adapter.cpp

#include "tun_adapter.h"
#include <wintun.h>

class TunAdapter {
public:
    TunAdapter() = default;
    ~TunAdapter() { Close(); }

    bool Create(const std::wstring& name) {
        // Load WinTUN library
        m_wintun = LoadLibraryW(L"wintun.dll");
        if (!m_wintun) {
            LogError("Failed to load wintun.dll");
            return false;
        }

        // Get function pointers
        auto WintunCreateAdapter = (WINTUN_CREATE_ADAPTER_FUNC*)
            GetProcAddress(m_wintun, "WintunCreateAdapter");

        // Create adapter
        GUID guid = { /* ... */ };
        m_adapter = WintunCreateAdapter(
            name.c_str(),
            L"LostLove",
            &guid
        );

        if (!m_adapter) {
            LogError("Failed to create WinTUN adapter");
            return false;
        }

        // Start session
        m_session = WintunStartSession(m_adapter, 0x400000);  // 4MB ring buffer

        return true;
    }

    void Close() {
        if (m_session) {
            WintunEndSession(m_session);
            m_session = nullptr;
        }

        if (m_adapter) {
            WintunCloseAdapter(m_adapter);
            m_adapter = nullptr;
        }

        if (m_wintun) {
            FreeLibrary(m_wintun);
            m_wintun = nullptr;
        }
    }

    std::optional<Packet> ReadPacket() {
        DWORD packet_size;
        BYTE* packet = WintunReceivePacket(m_session, &packet_size);

        if (!packet) {
            return std::nullopt;
        }

        Packet result;
        result.data.assign(packet, packet + packet_size);

        WintunReleaseReceivePacket(m_session, packet);

        return result;
    }

    bool WritePacket(const Packet& packet) {
        BYTE* buffer = WintunAllocateSendPacket(m_session, packet.data.size());
        if (!buffer) {
            return false;
        }

        std::memcpy(buffer, packet.data.data(), packet.data.size());

        WintunSendPacket(m_session, buffer);

        return true;
    }

    bool SetIPAddress(const std::string& ip, int prefix_length) {
        // Use netsh or IP Helper API
        std::wstring cmd = L"netsh interface ip set address \"" +
                          m_name +
                          L"\" static " +
                          StringToWString(ip) +
                          L" " +
                          StringToWString(PrefixToNetmask(prefix_length));

        return ExecuteCommand(cmd);
    }

private:
    HMODULE m_wintun = nullptr;
    WINTUN_ADAPTER_HANDLE m_adapter = nullptr;
    WINTUN_SESSION_HANDLE m_session = nullptr;
    std::wstring m_name;
};
```

### 4. Auto-Reconnect Logic

```cpp
// service/src/connection/auto_reconnect.cpp

class AutoReconnect {
public:
    AutoReconnect(ConnectionManager* manager)
        : m_manager(manager)
    {
        m_thread = std::thread([this]() {
            MonitorThread();
        });
    }

    ~AutoReconnect() {
        m_running = false;
        if (m_thread.joinable()) {
            m_thread.join();
        }
    }

    void Enable(bool enabled) {
        m_enabled = enabled;
    }

private:
    void MonitorThread() {
        while (m_running) {
            if (m_enabled && !m_manager->IsConnected() && m_should_reconnect) {
                LogInfo("Auto-reconnecting...");

                // Exponential backoff
                int delay = std::min(m_retry_delay, 60);
                std::this_thread::sleep_for(std::chrono::seconds(delay));

                if (m_manager->Connect(m_last_config)) {
                    LogInfo("Reconnected successfully");
                    m_retry_delay = 1;
                }
                else {
                    LogError("Reconnection failed, retrying...");
                    m_retry_delay *= 2;
                }
            }

            std::this_thread::sleep_for(std::chrono::seconds(1));
        }
    }

private:
    ConnectionManager* m_manager;
    std::thread m_thread;

    std::atomic<bool> m_running{ true };
    std::atomic<bool> m_enabled{ true };
    std::atomic<bool> m_should_reconnect{ false };

    ServerConfig m_last_config;
    int m_retry_delay = 1;
};
```

## Features

### 1. Split Tunneling

```cpp
class SplitTunneling {
public:
    void AddRoute(const std::string& cidr, bool use_vpn) {
        if (use_vpn) {
            m_vpn_routes.push_back(cidr);
        }
        else {
            m_direct_routes.push_back(cidr);
        }

        UpdateRoutingTable();
    }

    void AddApplication(const std::wstring& exe_path, bool use_vpn) {
        if (use_vpn) {
            m_vpn_apps.push_back(exe_path);
        }
        else {
            m_direct_apps.push_back(exe_path);
        }

        UpdateFirewallRules();
    }

private:
    void UpdateRoutingTable() {
        // Modify Windows routing table
        for (const auto& route : m_vpn_routes) {
            AddRoute(route, m_tun_interface);
        }

        for (const auto& route : m_direct_routes) {
            AddRoute(route, m_default_interface);
        }
    }

    void UpdateFirewallRules() {
        // Create Windows Firewall rules per application
        for (const auto& app : m_vpn_apps) {
            CreateFirewallRule(app, m_tun_interface);
        }
    }
};
```

### 2. Kill Switch

```cpp
class KillSwitch {
public:
    void Enable() {
        if (m_enabled) return;

        LogInfo("Enabling kill switch");

        // Block all traffic except to VPN server
        BlockAllTraffic();
        AllowVPNServer(m_server_ip);

        m_enabled = true;
    }

    void Disable() {
        if (!m_enabled) return;

        LogInfo("Disabling kill switch");

        // Restore normal traffic
        RemoveAllRules();

        m_enabled = false;
    }

    void OnConnectionLost() {
        if (m_enabled) {
            LogWarning("Connection lost, kill switch active - blocking all traffic");
            BlockAllTraffic();
        }
    }

private:
    void BlockAllTraffic() {
        // Windows Firewall rule: block all outbound
        std::wstring cmd = L"netsh advfirewall firewall add rule "
                          L"name=\"LostLove KillSwitch\" "
                          L"dir=out action=block";

        ExecuteCommand(cmd);
    }

    void AllowVPNServer(const std::string& server_ip) {
        // Allow traffic to VPN server
        std::wstring cmd = L"netsh advfirewall firewall add rule "
                          L"name=\"LostLove VPN Server\" "
                          L"dir=out action=allow "
                          L"remoteip=" + StringToWString(server_ip);

        ExecuteCommand(cmd);
    }

    bool m_enabled = false;
    std::string m_server_ip;
};
```

## Configuration

```json
// config.json

{
  "general": {
    "auto_connect": true,
    "auto_reconnect": true,
    "start_minimized": false,
    "launch_on_startup": true
  },

  "connection": {
    "protocol": "auto",
    "port": 443,
    "timeout": 30,
    "keep_alive_interval": 10
  },

  "obfuscation": {
    "enabled": true,
    "mode": "adaptive",
    "primary_disguise": "video_streaming",
    "dpi_evasion": "aggressive"
  },

  "crypto": {
    "mode": "maximum_security",
    "ecc_enabled": true,
    "hse_enabled": true,
    "qrl_enabled": true
  },

  "network": {
    "ipv6": true,
    "dns": "automatic",
    "custom_dns": [],
    "mtu": 1400
  },

  "features": {
    "kill_switch": true,
    "split_tunneling": {
      "enabled": false,
      "routes": [],
      "applications": []
    },
    "leak_protection": true
  },

  "ui": {
    "theme": "dark",
    "language": "en",
    "show_notifications": true,
    "minimize_to_tray": true
  }
}
```

## Installation & Deployment

### Build Requirements

```
- Visual Studio 2022 (C++ tools)
- Node.js 18+
- CMake 3.20+
- WinTUN SDK
- vcpkg for dependencies
```

### Build Process

```powershell
# Install dependencies
vcpkg install openssl:x64-windows
vcpkg install nlohmann-json:x64-windows
vcpkg install spdlog:x64-windows

# Build service
cd client/desktop-windows/service
mkdir build && cd build
cmake .. -DCMAKE_TOOLCHAIN_FILE=[vcpkg-root]/scripts/buildsystems/vcpkg.cmake
cmake --build . --config Release

# Build GUI
cd ../../gui
npm install
npm run build

# Create installer
cd ../installer
makensis installer.nsi
```

### Installation

```
LostLove-Setup.exe:
  1. Extract files to Program Files
  2. Install WinTUN driver
  3. Register Windows Service
  4. Create Start Menu shortcuts
  5. Configure auto-start
  6. Add firewall exceptions
```

## Security

1. **Privilege Separation**: GUI runs as user, service as SYSTEM
2. **Code Signing**: All executables signed with EV certificate
3. **Driver Verification**: WinTUN driver signature verification
4. **Secure IPC**: Encrypted communication between GUI and service
5. **No credential storage**: Passwords not stored locally
6. **Memory protection**: Sensitive data wiped from memory
7. **Anti-debugging**: Detects debugging attempts

## System Requirements

```
Minimum:
- Windows 10 (64-bit) version 1809 or later
- 2 GB RAM
- 100 MB disk space
- Administrator rights for installation

Recommended:
- Windows 11 (64-bit)
- 4 GB RAM
- 200 MB disk space
- SSD for better performance
```
