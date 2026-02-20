export async function instantiateWasm(modulePath) {
	let wasmInstance = null;
	const importObject = {
		'env': {
			'write_to_console': (ptr, len) => {
				const mem = wasmInstance.exports['memory'].buffer;
				const str = new TextDecoder().decode(new Uint8Array(mem, ptr, len));
				console.log(str);
			},
			'write_to_file': (namePtr, nameLen, bytesPtr, bytesLen) => {
				const mainMemory = wasmInstance.exports['memory'];
				const bytes = new Uint8Array(mainMemory.buffer, bytesPtr, bytesLen);
				const name = new TextDecoder().decode(new Uint8Array(mainMemory.buffer, namePtr, nameLen));
				const blob = new Blob([bytes], { type: "application/octet-stream" });
				const url = URL.createObjectURL(blob);
				const a = document.createElement('a');
				a.href = url;
				a.download = name;
				document.body.appendChild(a);
				a.click();
				document.body.removeChild(a);
				URL.revokeObjectURL(url);
			},
			'instantiate_module': (ptr, len) => {
				try {
					console.log("instantiating module", ptr, len);

					const mainMemory = wasmInstance.exports['memory'];
					const auxModuleBuffer = new Uint8Array(mainMemory.buffer, ptr, len);

					// instantiate the auxiliary module
					let auxModule = new WebAssembly.Module(auxModuleBuffer);
					new WebAssembly.Instance(auxModule, {
						'env': {
							...wasmInstance.exports
						}
					});

					return true;
				} catch (e) {
					console.error(e);
					return false;
				}
			},
			'grow_function_table': (numSlots) => {
				console.log("growing function table by ", numSlots);
				const mainTable = wasmInstance.exports['__indirect_function_table'];
				try {
					const latest_index = mainTable.grow(numSlots);
					return latest_index;
				} catch (e) {
					console.error(e);
					return -1;
				}
			},
			'visualize_update': (bytesPtr, bytesLen) => {
				console.log("module tried to visualize update", bytesPtr, bytesLen);
			}
		}
	};
	const wasmResult = await WebAssembly.instantiateStreaming(fetch(modulePath), importObject);
	wasmInstance = wasmResult.instance;
	return wasmInstance;
}

async function hello() {
	console.log("hello");
}
