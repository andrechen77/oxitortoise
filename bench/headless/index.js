import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// parse and validate command line arguments
function usage() {
	console.error("Usage: node index.js <oxitortoise|tortoise> <model_name>");
	process.exit(1);
}
if (process.argv.length !== 4) usage();
const mode = process.argv[2];
if (mode !== "oxitortoise" && mode !== "tortoise") usage();
const modelName = process.argv[3];

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const LOCAL_ROOT = path.join(__dirname, '..', '..'); // points to the project root

// launch the browser and create a new page
const browser = await puppeteer.launch();
const page = await browser.newPage();

// capture console messages from the page
page.on('console', msg => {
	console.log(`PAGE: ${msg.text().trimEnd()}`);
});

// intercept requests
page.setRequestInterception(true);
page.on('request', request => {
	if (request.isInterceptResolutionHandled()) return;
	const url = new URL(request.url());
	// localhost:8000 serves local files from the project root
	if (url.hostname === 'localhost' && url.port === '8000') {
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
			request.respond({ status: 404, body: `Error: ${e}`, headers: { "Access-Control-Allow-Origin": "*" } });
		}
	} else {
		request.continue();
	}
});
page.on('requestfailed', (request) => {
	const url = new URL(request.url());
	console.log('Request to', request.url(), 'failed with reason:', request.failure()?.errorText);
	if (url.hostname === 'localhost' && url.port === '9000') {
		console.log('Request to localhost:9000 is intended for Galapagos. Make sure an instance of Galapagos is running on port 9000.');
	}
});

switch (mode) {
	case "oxitortoise":
		await runOxitortoise(modelName);
		break;
	case "tortoise":
		await runTortoise(modelName);
		break;
}

async function runOxitortoise(modelName) {
	const wasmRunnerPath = "http://localhost:8000/bench/oxitortoise_runner.js";
	await page.evaluate(async (wasmRunnerPath, modelName) => {
		const { instantiateWasm } = await import(wasmRunnerPath)

		let modulePath = `http://localhost:8000/bench/models/${modelName}/run.wasm`;
		let wasmInstance = await instantiateWasm(modulePath, update => console.log(`module tried to visualize update of length ${update.length}`));
		console.log("running main()");
		wasmInstance.exports['main']();
		console.log("running perf trials");
		console.time("perf_trials");
		wasmInstance.exports['perf_trials']();
		console.timeEnd("perf_trials");
	}, wasmRunnerPath, modelName);
}

async function runTortoise(modelName) {
	await page.addScriptTag({ url: "http://localhost:9000/netlogo-engine.js" });
	await page.addScriptTag({ url: "http://localhost:9000/tortoise-compiler.js" });
	await page.evaluate(async (modelName) => {
		const { compileModel, runPerfTrials } = await import("http://localhost:8000/bench/tortoise_runner.js");
		await compileModel(`http://localhost:8000/bench/models/${modelName}/model.nlogox`);
		console.log(`compiled model ${modelName}`);

		console.log("running perf trials");
		console.time("perf_trials");
		runPerfTrials();
		console.timeEnd("perf_trials");
	}, modelName);
}


await browser.close();
