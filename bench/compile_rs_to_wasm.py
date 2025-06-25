#!/usr/bin/env python3
"""
Script to compile Rust projects to WebAssembly and copy results to model directories.
"""

import os
import subprocess
import shutil
import sys
from pathlib import Path


def run_command(cmd, cwd=None):
    """Run a command and return the result."""
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            shell=True,
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
        print(f"Error: {e.stderr}")
        return None


def compile_models_to_wasm():
    """Compile all .rs models to Wasm."""
    # Get the current working directory (should be the project root)
    project_root = Path.cwd()
    models_dir = project_root / "bench" / "models"

    if not models_dir.exists():
        print(f"Models directory not found: {models_dir}")
        return False

    print(f"Looking for subdirectories in: {models_dir}")

    # Find all subdirectories in the models directory
    subdirs = [d for d in models_dir.iterdir() if d.is_dir()]

    if not subdirs:
        print("No subdirectories found in models directory")
        return False

    print(f"Found {len(subdirs)} subdirectories: {[d.name for d in subdirs]}")

    success_count = 0

    for subdir in subdirs:
        subdir_name = subdir.name
        print(f"\nProcessing: {subdir_name}")

		# build the wasm module
        build_cmd = f"cargo build --bin {subdir_name} --release --target wasm32-unknown-unknown"
        print(f"Running: {build_cmd}")
        result = run_command(build_cmd, cwd=project_root)
        if result is None:
            print(f"Failed to build {subdir_name}")
            continue

        # copy the wasm module to the models directory
        artifact_path = project_root / "target" / "wasm32-unknown-unknown" / "release" / f"{subdir_name}.wasm"
        artifact_dest = subdir / "wasm.wasm"
        if not artifact_path.exists():
            print(f"Warning: Expected WASM file not found: {artifact_path}")
            continue
        try:
            shutil.copy2(artifact_path, artifact_dest)
            print(f"Copied {artifact_path} to {artifact_dest}")
            success_count += 1
        except Exception as e:
            print(f"Failed to copy WASM file for {subdir_name}: {e}")

    print(f"\nCompilation complete. Successfully processed {success_count}/{len(subdirs)} subdirectories.")
    return success_count == len(subdirs)


if __name__ == "__main__":
    success = compile_models_to_wasm()
    sys.exit(0 if success else 1)
