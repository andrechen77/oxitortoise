export async function compileModel(modelFilePath) {
	// fetch the model's .nlogox file
	const nlogox = await fetch(modelFilePath).then(r => r.text());

	// compile the model into JS code
	const compileResult = new BrowserCompiler().fromNlogoXML(nlogox, [], { code: "", widgets: [] });
	if (!compileResult.model.success) {
		throw new Error(`failed to compile model ${modelName}`);
	}

	// load the model code as a script
	const modelCode = compileResult.model.result;
	const blobUrl = URL.createObjectURL(new Blob([modelCode], { type: 'text/javascript' }));
	const script = document.createElement('script');
	script.src = blobUrl;
	script.async = true;
	document.body.appendChild(script);
	await new Promise((resolve, reject) => {
		script.onload = () => resolve();
		script.onerror = () => reject(new Error(`Failed to load script at ${script.src}`))
	});
}

export function runPerfTrials() {
	for (let trial = 0; trial < 100; trial++) {
		window.ProcedurePrims.callCommand("setup")
		for (let tick = 0; tick < 1000; tick++) {
			window.ProcedurePrims.callCommand("go")
		}
	}
}