const mockSubscription = `vless://46019159-ea02-4f60-a25f-3bde50783d57@aws-link1.liangxin1.xyz:35248?type=tcp&encryption=none&host=&path=&headerType=none&quicSecurity=none&serviceName=&security=reality&flow=xtls-rprx-vision&fp=chrome&sni=www.lamer.com.hk&pbk=IGsSxC0wgn7wLy0NM0QN_yOREDKT_814Y_3_rbgDoTc&sid=c8c0f951#Proxy_1
vless://46019159-ea02-4f60-a25f-3bde50783d57@aws-link1.liangxin1.xyz:35248?type=tcp&encryption=none&host=&path=&headerType=none&quicSecurity=none&serviceName=&security=reality&flow=xtls-rprx-vision&fp=chrome&sni=www.lamer.com.hk&pbk=IGsSxC0wgn7wLy0NM0QN_yOREDKT_814Y_3_rbgDoTc&sid=c8c0f951#Proxy_2`;

const encoded = btoa(mockSubscription);

const _server = Bun.serve({
	port: 3001,
	fetch(_req) {
		return new Response(encoded);
	},
});

console.log("Mock subscription server running on port 3001");

// Now verify the main server
async function test() {
	console.log("Testing converter...");
	const response = await fetch(
		"http://localhost:3000/?url=http://localhost:3001/sub",
	);
	if (!response.ok) {
		console.error("Test failed:", response.status, await response.text());
		process.exit(1);
	}

	const text = await response.text();
	console.log("Surge Response:\n", text);

	const fs = require("node:fs");
	const config = JSON.parse(fs.readFileSync("xray.json", "utf8"));
	console.log("Xray Config:\n", JSON.stringify(config, null, 2));

	process.exit(0);
}

// Give the main server a second to start if we were running it, but we assume it's running separately or we start it here?
// Ideally we run this script and it tests the other one.
test();
