# xray2surge

A lightweight tool to convert VLESS subscription links into a format compatible with Surge Mac, utilizing [Xray-core](https://github.com/XTLS/Xray-core) as an external proxy.

This tool solves the problem of Surge not natively supporting VLESS/REALITY protocols by bridging it with Xray.

## Prerequisites

- **[Bun](https://bun.sh/)**: The Javascript runtime used to run this tool.
- **[Xray-core](https://github.com/XTLS/Xray-core)**: The core executable for handling VLESS connections.
## Configuration

You can configure the application using environment variables. Copy the `.env.example` file to `.env` and modify the values as needed:

```bash
cp .env.example .env
```

Available variables:

- `XRAY_PATH`: Path to the Xray executable (default: `/opt/homebrew/bin/xray`)
- `CONFIG_FILE_NAME`: Name of the generated Xray config file (default: `xray.json`)
- `CONFIG_PATH`: Full path to the generated Xray config file (optional, overrides `CONFIG_FILE_NAME` + `cwd`)
- `SOCKS_START_PORT`: Starting port for SOCKS5 proxies (default: `50000`)
- `SERVER_PORT`: Port for this conversion server (default: `3123`)
- `DEFAULT_USER_LEVEL`: Xray user level (default: `8`)
- `DEFAULT_FINGERPRINT`: TLS/Reality fingerprint (default: `safari`)
- `LOOPBACK_ADDRESS`: Loopback address to bind (default: `127.0.0.1`)
- `SUBSCRIPTION_USER_AGENT`: User-Agent for fetching subscriptions (default: `v2rayNG/1.8.5`)
- `FILTER_KEYWORDS`: Comma-separated keywords to filter out proxies by name

## Usage

### 1. Start the Server

Run the conversion server:

```bash
bun run index.ts
```

The server will start on `http://localhost:3123` (or your configured `SERVER_PORT`).

### 2. Convert Subscription

Open your browser or use curl to fetch the Surge configuration from your VLESS subscription URL:

```
http://localhost:3123/?url=<YOUR_VLESS_SUBSCRIPTION_URL>
```

*Note: The tool expects the subscription URL to return a base64 encoded list of `vless://` links.*

### 3. Configure Surge

The response will be a list of proxy definitions compatible with Surge.

1. **Copy** the output text.
2. **Paste** it into the `[Proxy]` section of your Surge configuration file.

The output will look something like this:

```ini
Proxy_Name_1 = external, exec = "/opt/homebrew/bin/xray", local-port = 50001, args = "run", args = "-c", args = "/path/to/xray2surge/xray.json"
Proxy_Name_2 = socks5, 127.0.0.1, 50001
...
```

### How it Works

1. The script fetches and parses your VLESS subscription.
2. It generates a valid `xray.json` configuration configures Xray to listen on local SOCKS5 ports (starting from 50000), forwarding traffic to the remote VLESS servers.
3. It generates Surge proxy definitions:
    - The **first proxy** in the list is defined as an `external` proxy. This tells Surge to launch the Xray process using the generated `xray.json`.
    - Subsequent proxies are defined as `socks5` proxies pointing to the local ports that Xray is listening on.
4. When Surge starts (or when you activate the proxy), it runs Xray in the background. Surge then routes traffic to the local SOCKS5 ports, which Xray handles and tunnels via VLESS.
