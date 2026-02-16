#!/bin/bash

set -m
trap "pkill -P $$" EXIT INT TERM

# Check the first argument for build mode (release or dev/empty)
# If the argument is "release", build in release mode and set PROFILE=release
# Otherwise, build in development mode and set PROFILE=debug
if [[ "$1" == "release" ]]; then
    PROFILE=release
    BUILD_FLAGS="--release"
    echo "Building in release mode"
else
    PROFILE=debug
    BUILD_FLAGS=""
    echo "Building in development mode"
fi

MODEL_NAME=ants
TARGET=wasm32-unknown-unknown

RUSTFLAGS="--remap-path-prefix=$PWD=../../.." cargo build --bin $MODEL_NAME --target $TARGET $BUILD_FLAGS

mv ../target/$TARGET/$PROFILE/$MODEL_NAME.wasm models/$MODEL_NAME/run.wasm

echo "starting server, go to http://localhost:8000/wasm_runner.html or http://localhost:8000/wasm_runner_visualizer.html"

# running a server is necessary to avoid CORS issues
python3 -m http.server 8000 &

# start an sbt server to serve Galapagos
cd galapagos && sbt start &

wait
