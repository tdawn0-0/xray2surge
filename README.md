# xray2surge

A lightweight tool to convert proxy subscription links into a format compatible with Surge Mac, utilizing [Xray-core](https://github.com/XTLS/Xray-core) as an external proxy.

This tool solves the problem of Surge not natively supporting VLESS/REALITY and Hysteria2 protocols by bridging them with Xray.

## Features

- **VLESS/REALITY Support**: Full support for VLESS protocol with REALITY transport
- **Hysteria2 Support**: Native Hysteria2 proxy support with port hopping
- **Multiple Subscriptions**: Merge proxies from multiple subscription URLs
- **Auto Deduplication**: Automatically handles duplicate proxy names by appending hostnames
- **Keyword Filtering**: Filter out unwanted proxies by name keywords

## Prerequisites

- **[Rust](https://www.rust-lang.org/)**: Required if you want to build from source.
- **[Xray-core](https://github.com/XTLS/Xray-core)**: The core executable for handling VLESS connections.

## Configuration

You can configure the application using environment variables. Copy the `.env.example` file to `.env` and modify the values as needed:

```bash
cp .env.example .env
```

Available variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `XRAY_PATH` | `/opt/homebrew/bin/xray` | Path to the Xray executable |
| `CONFIG_FILE_NAME` | `xray.json` | Name of the generated Xray config file |
| `CONFIG_PATH` | `<cwd>/xray.json` | Full path to the generated Xray config file |
| `SOCKS_START_PORT` | `50000` | Starting port for SOCKS5 proxies |
| `SERVER_PORT` | `3123` | Port for this conversion server |
| `DEFAULT_USER_LEVEL` | `8` | Xray user level |
| `DEFAULT_FINGERPRINT` | `safari` | TLS/Reality fingerprint |
| `LOOPBACK_ADDRESS` | `127.0.0.1` | Loopback address to bind |
| `SUBSCRIPTION_USER_AGENT` | `v2rayNG/1.8.5` | User-Agent for fetching subscriptions |
| `FILTER_KEYWORDS` | *(empty)* | Comma-separated keywords to filter out proxies |

## Usage

### 1. Build and Start the Server

Build the project in release mode for optimal performance:

```bash
cargo build --release
./target/release/xray2surge
```

Or run directly with Cargo:

```bash
cargo run --release
```

The server will start on `http://localhost:3123` (or your configured `SERVER_PORT`).

### Run with PM2 (Recommended for Production)

If you have PM2 installed, use the provided ecosystem file:

```bash
pm2 start ecosystem.config.cjs
```

### 2. Convert Subscription

**Single subscription:**

```
http://localhost:3123/?url=<YOUR_SUBSCRIPTION_URL>
```

**Multiple subscriptions** (comma-separated):

```
http://localhost:3123/?url=<URL1>,<URL2>,<URL3>
```

> **Note:** The tool expects subscription URLs to return base64-encoded lists of `vless://` or `hysteria2://` (or `hy2://`) links.

### 3. Configure Surge

The response will be a list of proxy definitions compatible with Surge.

1. **Copy** the output text.
2. **Paste** it into the `[Proxy]` section of your Surge configuration file.

**Example output:**

```ini
# VLESS proxies (via Xray)
Proxy_Name_1 = external, exec = "/opt/homebrew/bin/xray", local-port = 50001, args = "run", args = "-c", args = "/path/to/xray.json"
Proxy_Name_2 = socks5, 127.0.0.1, 50002

# Hysteria2 proxies (native Surge support)
HY2_Proxy = hysteria2, example.com, 443, password=xxx, sni=example.com
```

## How it Works

1. The app fetches and parses your subscriptions (VLESS and Hysteria2).
2. For VLESS proxies:
   - Generates a valid `xray.json` configuration with SOCKS5 inbound ports (starting from 50000)
   - Creates Surge `external` and `socks5` proxy definitions
3. For Hysteria2 proxies:
   - Directly generates native Surge `hysteria2` proxy definitions (no Xray needed)
4. When Surge activates the proxies, it runs Xray in the background for VLESS traffic while using native support for Hysteria2.

## Supported Protocols

| Protocol | Transport | Notes |
|----------|-----------|-------|
| VLESS | TCP, WebSocket | Requires Xray-core |
| VLESS + REALITY | TCP | Requires Xray-core |
| Hysteria2 | QUIC | Native Surge support, with port hopping |

## License

MIT
