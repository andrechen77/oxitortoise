#!/bin/bash

RUSTFLAGS="--remap-path-prefix=$PWD=../../.." cargo build --bin ants --target wasm32-unknown-unknown
mv target/wasm32-unknown-unknown/debug/ants.wasm bench/models/ants/run.wasm

echo "starting server, go to http://localhost:8000/bench/wasm_runner.html or http://localhost:8000/bench/wasm_runner_visualizer.html"

# running a server is necessary to avoid CORS issues
python3 -m http.server 8000