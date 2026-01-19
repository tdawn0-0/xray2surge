import { join } from "node:path";

const XRAY_PATH = "/opt/homebrew/bin/xray";
// We will generate the xray config file in the current directory or a specific surge directory if needed.
// For now let's put it in the current working directory to be safe and portable for this script.
const CONFIG_FILE_NAME = "xray.json";
const CONFIG_PATH = join(process.cwd(), CONFIG_FILE_NAME);

interface VlessConfig {
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
}

function parseVlessLink(link: string): VlessConfig | null {
	if (!link.startsWith("vless://")) return null;

	try {
		const url = new URL(link);
		const uuid = url.username;
		const [address, portStr] = url.host.split(":");

		if (!address || !portStr) {
			return null;
		}

		const port = parseInt(portStr, 10);
		const params = url.searchParams;

		return {
			uuid,
			address,
			port,
			type: params.get("type") || "tcp",
			encryption: params.get("encryption") || "none",
			security: params.get("security") || "",
			flow: params.get("flow") || "",
			sni: params.get("sni") || "",
			pbk: params.get("pbk") || "",
			sid: params.get("sid") || "",
			fp: params.get("fp") || "",
			name: decodeURIComponent(url.hash.substring(1)) || "proxy",
		};
	} catch (e) {
		console.error("Failed to parse link:", link, e);
		return null;
	}
}

interface XrayOutbound {
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
				fingerprint: string;
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
		};
	};
}

function generateXrayConfig(configs: VlessConfig[]): any {
	const inbounds = configs.map((_cfg, index) => ({
		tag: `proxy-${index}-in`,
		port: 40000 + index + 1, // Start from 40001
		listen: "127.0.0.1",
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
								fingerprint: cfg.fp,
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
		const port = 40000 + index + 1;
		// Sanitize name for Surge
		const name = cfg.name.trim().replace(/[\s,]+/g, "_");

		if (index === 0) {
			// First one is the external one that starts Xray
			// Note: We need to escape quotes for the args
			output += `${name} = external, exec = "${XRAY_PATH}", local-port = ${port}, args = "run", args = "-c", args = "${CONFIG_PATH}"\n`;
		} else {
			output += `${name} = socks5, 127.0.0.1, ${port}\n`;
		}
	});

	return output;
}

const server = Bun.serve({
	port: 3000,
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
					"User-Agent": "v2rayNG/1.8.5", // Use a common v2ray client UA to ensure we get valid links
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

			for (const link of links) {
				const config = parseVlessLink(link);
				if (config) {
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
