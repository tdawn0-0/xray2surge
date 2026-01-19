# xray2surge

A lightweight tool to convert VLESS subscription links into a format compatible with Surge Mac, utilizing [Xray-core](https://github.com/XTLS/Xray-core) as an external proxy.

This tool solves the problem of Surge not natively supporting VLESS/REALITY protocols by bridging it with Xray.

## Prerequisites

- **[Bun](https://bun.sh/)**: The Javascript runtime used to run this tool.
- **[Xray-core](https://github.com/XTLS/Xray-core)**: The core executable for handling VLESS connections.
  - Default expected path: `/opt/homebrew/bin/xray`
  - Ensure `xray` is installed and accessible at this path, or modify `XRAY_PATH` in `index.ts`.

## Installation

1. Clone this repository.
2. Install dependencies:

```bash
bun install
```

## Usage

### 1. Start the Server

Run the conversion server:

```bash
bun run index.ts
```

The server will start on `http://localhost:3000`.

### 2. Convert Subscription

Open your browser or use curl to fetch the Surge configuration from your VLESS subscription URL:

```
http://localhost:3000/?url=<YOUR_VLESS_SUBSCRIPTION_URL>
```

*Note: The tool expects the subscription URL to return a base64 encoded list of `vless://` links.*

### 3. Configure Surge

The response will be a list of proxy definitions compatible with Surge.

1. **Copy** the output text.
2. **Paste** it into the `[Proxy]` section of your Surge configuration file.

The output will look something like this:

```ini
Proxy_Name_1 = external, exec = "/opt/homebrew/bin/xray", local-port = 40001, args = "run", args = "-c", args = "/path/to/xray2surge/xray.json"
Proxy_Name_2 = socks5, 127.0.0.1, 40001
...
```

### How it Works

1. The script fetches and parses your VLESS subscription.
2. It generates a valid `xray.json` configuration configures Xray to listen on local SOCKS5 ports (starting from 40001), forwarding traffic to the remote VLESS servers.
3. It generates Surge proxy definitions:
    - The **first proxy** in the list is defined as an `external` proxy. This tells Surge to launch the Xray process using the generated `xray.json`.
    - Subsequent proxies are defined as `socks5` proxies pointing to the local ports that Xray is listening on.
4. When Surge starts (or when you activate the proxy), it runs Xray in the background. Surge then routes traffic to the local SOCKS5 ports, which Xray handles and tunnels via VLESS.
