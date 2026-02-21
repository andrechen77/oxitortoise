#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: $0 <model_name> [release]"
    exit 1
fi
MODEL_NAME=$1

# Check the first argument for build mode (release or dev/empty)
# If the argument is "release", build in release mode and set PROFILE=release
# Otherwise, build in development mode and set PROFILE=debug

if [[ "$2" == "release" ]]; then
    PROFILE=release
    BUILD_FLAGS="--release"
    echo "Building in release mode"
else
    PROFILE=debug
    BUILD_FLAGS=""
    echo "Building in development mode"
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PROJECT_ROOT=$(dirname $SCRIPT_DIR)

MODEL_NAME=ants
TARGET=wasm32-unknown-unknown

RUSTFLAGS="--remap-path-prefix=$PROJECT_ROOT=../../.." cargo build --bin $MODEL_NAME --target $TARGET $BUILD_FLAGS

mv $PROJECT_ROOT/target/$TARGET/$PROFILE/$MODEL_NAME.wasm $SCRIPT_DIR/models/$MODEL_NAME/run.wasm
