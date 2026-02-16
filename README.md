`oxitortoise` is a reimplementation of the NetLogo engine in Rust and
WebAssembly.

This document contains instructions for how to build and run the prototype.

# building and running a test instance of the Ants model

`oxitortoise` compiles NetLogo models into WebAssembly code which runs in the
same environment as the `oxitortoise` engine.

[`run.rs`](/bench/models/ants/run.rs) is currently the core of the prototype.
You can think of this as a script that pulls in components from the rest of the
project: it exists only to drive the compiler, as there is not yet a proper
user-facing interface for the compiler.

`run.rs` does a couple of things:
- Includes a hardcoded JSON representation of the AST of the "Ants" model from a
  [file](/bench/models/ants/ast.json) is loaded. This JSON was obtained using
  [nl2ast](https://github.com/NetLogo/nl2ast/tree).
- The AST is converted into the compiler's internal program representation,
  called MIR.
- A [`cheats.json`](/bench/models/ants/cheats.json) file is read and parsed;
  this is used to inject information into the compiler pipeline that the
  compiler currently does not yet have algorithms for. This information is added
  to the MIR.
- A variety of transformations on the MIR is run to simplify and lower it until
  it can be converted into LIR.
- The transformed MIR is converted into LIR (another internal representation)
  and then turned into WebAssembly.
- The generated WebAssembly is dynamically instantiated (i.e. hot-loaded as a
  JIT would do).

A script exists at `/bench/run.sh` to build and debug `run.rs`. When in the
browser page to run the file, press "Load" to load the Wasm module representing
the `run.rs` script, and "main()" to run the main function once it is loaded.
The module will contain debug info with source maps. I have found that using
Google Chrome with the ["C/C++ DevTools Support
(DWARF)"](https://chromewebstore.google.com/detail/cc++-devtools-support-dwa/pdcpmagijalfljmkmjngeonclgbbannb)
extension is the best way to get these to work.

When running `run.rs`, it might try to download intermediate compiler artifacts
for debugging purposes. You can use
[`wasm-tools`](https://github.com/bytecodealliance/wasm-tools) to read the Wasm
artifacts and any DOT graph visualizer for the MIR graphs.

