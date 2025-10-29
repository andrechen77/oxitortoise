`oxitortoise` is a reimplementation of the NetLogo engine in Rust and
WebAssembly.

This document contains instructions for how to build and run the prototype.

# building and running the compiler

`oxitortoise` compiles NetLogo models into WebAssembly code which runs in the
same environment as the `oxitortoise` engine.

To run the compiler on the "Ants" model, run [this script](/bench/models/ants/run.sh).
This should produce debugging logs in the terminal as well as generate images
of the compiler's internal representation in an `output` folder.

The core of this script is a `cargo run` command which will run the code in
[`run.rs`](/bench/models/ants/run.rs). This file exists only to drive the
compiler, as there is not yet a proper user-facing interface for the compiler.

`run.rs` does a couple of things:
- A JSON representation of the AST of the "Ants" model from a
  [file](/bench/models/ants/ast.json) is loaded. This JSON was obtained using
  [nl2ast](https://github.com/NetLogo/nl2ast/tree).
- The AST is converted into the compiler's internal program representation,
  called MIR.
- A [`cheats.json`](/bench/models/ants/cheats.json) file is read and parsed;
  this is used to inject information into the compiler pipeline that the
  compiler currently does not yet have algorithms for (e.g. the types of
  variables). This information is added to the MIR.
- A variety of transformations on the MIR is run to simplify and lower it until
  it can be converted into WebAssembly.
- (currently unimplemented) The transformed MIR is converted into LIR (another
  internal representation) and then turned into WebAssembly.

