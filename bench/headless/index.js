import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

console.log("Hello world!");

const browser = await puppeteer.launch();
const page = await browser.newPage();

// points to the project root
const LOCAL_ROOT = path.join(__dirname, '..', '..');

// capture console messages from the page
page.on('console', msg => {
	console.log(`PAGE: ${msg.text().trimEnd()}`);
});

// intercept certain requests
page.setRequestInterception(true);
page.on('request', request => {
	if (request.isInterceptResolutionHandled()) return;
	console.log("intercepted request for", request.url());
	const url = new URL(request.url());
	if (url.hostname === 'localhost') {
		// Map /some/path -> LOCAL_ROOT/some/path
		const relativePath = url.pathname.replace(/^\//, ""); // remove leading slash
		const localPath = path.join(LOCAL_ROOT, relativePath);

		// fulfill the request with the local file
		try {
			const body = fs.readFileSync(localPath);
			// guess content type by file extension
			let contentType = "application/octet-stream";
			if (localPath.endsWith(".js")) contentType = "application/javascript";
			else if (localPath.endsWith(".wasm")) contentType = "application/wasm";
			else if (localPath.endsWith(".html")) contentType = "text/html";
			else if (localPath.endsWith(".css")) contentType = "text/css";
			request.respond({
				status: 200,
				body,
				contentType,
				headers: {
					"Access-Control-Allow-Origin": "*",
				},
			});
		} catch (e) {
			request.respond({ status: 404, body: `Error: ${e}` });
		}
	} else {
		request.continue();
	}
});


const wasmRunnerPath = "http://localhost/bench/wasm_runner.js";
await page.evaluate(async (wasmRunnerPath) => {
	const { instantiateWasm } = await import(wasmRunnerPath)

	let modulePath = `http://localhost/bench/models/ants/run.wasm`;
	let wasmInstance = await instantiateWasm(modulePath);
	console.log("running main()");
	wasmInstance.exports['main']();
	console.log("running perf trials");
	console.time("perf_trials");
	wasmInstance.exports['perf_trials']();
	console.timeEnd("perf_trials");
}, wasmRunnerPath);

await browser.close();
