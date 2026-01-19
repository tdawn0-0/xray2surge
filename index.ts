import { join } from "node:path";

const XRAY_PATH = process.env.XRAY_PATH || "/opt/homebrew/bin/xray";
// We will generate the xray config file in the current directory or a specific surge directory if needed.
// For now let's put it in the current working directory to be safe and portable for this script.
const CONFIG_FILE_NAME = process.env.CONFIG_FILE_NAME || "xray.json";
const CONFIG_PATH =
	process.env.CONFIG_PATH || join(process.cwd(), CONFIG_FILE_NAME);

const SOCKS_START_PORT = parseInt(process.env.SOCKS_START_PORT || "50000", 10);
const SERVER_PORT = parseInt(process.env.SERVER_PORT || "3123", 10);
const DEFAULT_USER_LEVEL = parseInt(process.env.DEFAULT_USER_LEVEL || "8", 10);
const DEFAULT_FINGERPRINT = process.env.DEFAULT_FINGERPRINT || "safari";
const LOOPBACK_ADDRESS = process.env.LOOPBACK_ADDRESS || "127.0.0.1";
const SUBSCRIPTION_USER_AGENT =
	process.env.SUBSCRIPTION_USER_AGENT || "v2rayNG/1.8.5";
// 逗号分隔的关键词，用于过滤掉名称中包含这些关键词的 proxy
const FILTER_KEYWORDS = process.env.FILTER_KEYWORDS || "";

export interface VlessConfig {
	uuid: string;
	address: string;
	port: number;
	type: string;
	encryption: string;
	security: string;
	flow: string;
	sni: string;
	pbk: string;
	sid: string;
	fp: string;
	name: string;
	path: string;
	host: string;
}

export function parseVlessLink(link: string): VlessConfig | null {
	if (!link.startsWith("vless://")) return null;

	try {
		const url = new URL(link);
		const uuid = url.username;
		const [address, portStr] = url.host.split(":");

		if (!address || !portStr) {
			console.log("Invalid address or port", address, portStr);
			return null;
		}

		const port = parseInt(portStr, 10);
		const params = url.searchParams;

		const type = params.get("type") || "tcp";
		const security = params.get("security") || "";
		let flow = params.get("flow") || "";

		// Auto-fill flow for Reality/TLS + TCP if missing to avoid Xray warnings
		if (security === "reality" && !flow) {
			flow = "xtls-rprx-vision";
		}

		return {
			uuid,
			address,
			port,
			type,
			encryption: params.get("encryption") || "none",
			security,
			flow,
			sni: params.get("sni") || "",
			pbk: params.get("pbk") || "",
			sid: params.get("sid") || "",
			fp: params.get("fp") || "",
			name: decodeURIComponent(url.hash.substring(1)) || "proxy",
			path: params.get("path") || "",
			host: params.get("host") || "",
		};
	} catch (e) {
		console.error("Failed to parse link:", link, e);
		return null;
	}
}

export interface XrayOutbound {
	tag: string;
	protocol: string;
	settings: {
		vnext: Array<{
			address: string;
			port: number;
			users: Array<{
				id: string;
				flow: string;
				encryption: string;
				level: number;
			}>;
		}>;
	};
	streamSettings: {
		network: string;
		security: string;
		realitySettings?: {
			show: boolean;
			serverName: string;
			publicKey: string;
			shortId: string;
			fingerprint: string;
		};
		tlsSettings?: {
			allowInsecure: boolean;
			fingerprint: string;
			serverName: string;
			show: boolean;
		};
		wsSettings?: {
			headers: {
				Host: string;
			};
			path: string;
		};
	};
}

export function generateXrayConfig(configs: VlessConfig[]): any {
	const inbounds = configs.map((_cfg, index) => ({
		tag: `proxy-${index}-in`,
		port: SOCKS_START_PORT + index + 1, // Start from 50001
		listen: LOOPBACK_ADDRESS,
		protocol: "socks",
		settings: {
			auth: "none",
		},
	}));

	const outbounds = configs.map((cfg, index): XrayOutbound => {
		const outbound: XrayOutbound = {
			tag: `proxy-${index}-out`,
			protocol: "vless",
			settings: {
				vnext: [
					{
						address: cfg.address,
						port: cfg.port,
						users: [
							{
								id: cfg.uuid,
								flow: cfg.flow,
								encryption: cfg.encryption,
								level: DEFAULT_USER_LEVEL,
							},
						],
					},
				],
			},
			streamSettings: {
				network: cfg.type,
				security: cfg.security,
			},
		};

		if (cfg.security === "reality") {
			outbound.streamSettings.realitySettings = {
				show: false,
				serverName: cfg.sni,
				publicKey: cfg.pbk,
				shortId: cfg.sid,
				fingerprint: cfg.fp || DEFAULT_FINGERPRINT,
			};
		}

		if (cfg.security === "tls") {
			outbound.streamSettings.tlsSettings = {
				allowInsecure: false,
				fingerprint: cfg.fp || DEFAULT_FINGERPRINT,
				serverName: cfg.sni,
				show: false,
			};
		}

		if (cfg.type === "ws") {
			outbound.streamSettings.wsSettings = {
				headers: {
					Host: cfg.host,
				},
				path: cfg.path,
			};
		}

		return outbound;
	});

	const rules = configs.map((_, index) => ({
		type: "field",
		inboundTag: [`proxy-${index}-in`],
		outboundTag: `proxy-${index}-out`,
	}));

	return {
		inbounds,
		outbounds,
		routing: {
			rules,
		},
	};
}

function generateSurgeList(configs: VlessConfig[]): string {
	if (configs.length === 0) return "";

	let output = "[Proxy]\n";

	configs.forEach((cfg, index) => {
		const port = SOCKS_START_PORT + index + 1;
		// Sanitize name for Surge
		const name = cfg.name.trim().replace(/[\s,]+/g, "_");

		if (index === 0) {
			// First one is the external one that starts Xray
			// Note: We need to escape quotes for the args
			output += `${name} = external, exec = "${XRAY_PATH}", local-port = ${port}, args = "run", args = "-c", args = "${CONFIG_PATH}"\n`;
		} else {
			output += `${name} = socks5, ${LOOPBACK_ADDRESS}, ${port}\n`;
		}
	});

	return output;
}

if (import.meta.main) {
	const server = Bun.serve({
		port: SERVER_PORT,
		async fetch(req) {
			const url = new URL(req.url);
			const targetUrl = url.searchParams.get("url");

			if (!targetUrl) {
				return new Response("Missing 'url' query parameter", { status: 400 });
			}

			try {
				console.log(`Fetching subscription from: ${targetUrl}`);
				const response = await fetch(targetUrl, {
					headers: {
						"User-Agent": SUBSCRIPTION_USER_AGENT, // Use a common v2ray client UA to ensure we get valid links
					},
				});

				if (!response.ok) {
					return new Response(
						`Failed to fetch subscription: ${response.statusText}`,
						{ status: 500 },
					);
				}

				const encodedBody = await response.text();
				// Try to decode Base64
				let decodedBody: string;
				try {
					decodedBody = atob(encodedBody.trim());
				} catch (_e) {
					// Sometimes it might not be base64 encoded or have whitespace?
					// Let's assume standard subscription return is base64
					return new Response("Failed to decode base64 subscription body", {
						status: 500,
					});
				}

				const links = decodedBody
					.split("\n")
					.map((l) => l.trim())
					.filter((l) => l.length > 0);
				const vlessConfigs: VlessConfig[] = [];
				const filterWords = FILTER_KEYWORDS.split(",")
					.map((w) => w.trim())
					.filter((w) => w.length > 0);

				for (const link of links) {
					const config = parseVlessLink(link);
					if (config) {
						const isFiltered = filterWords.some((w) => config.name.includes(w));
						if (isFiltered) {
							console.log(`Filtered out proxy: ${config.name}`);
							continue;
						}
						vlessConfigs.push(config);
					}
				}

				if (vlessConfigs.length === 0) {
					return new Response("No valid VLESS links found", { status: 404 });
				}

				const xrayConfig = generateXrayConfig(vlessConfigs);

				// Write Xray config to file
				await Bun.write(CONFIG_PATH, JSON.stringify(xrayConfig, null, 2));
				console.log(`Written Xray config to ${CONFIG_PATH}`);

				const surgeList = generateSurgeList(vlessConfigs);

				return new Response(surgeList, {
					headers: {
						"Content-Type": "text/plain; charset=utf-8",
					},
				});
			} catch (error) {
				console.error(error);
				return new Response("Internal Server Error", { status: 500 });
			}
		},
	});

	console.log(`Listening on http://localhost:${server.port}`);
	console.log(`Xray Config will be saved to: ${CONFIG_PATH}`);
}
