use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if target == "wasm32-unknown-unknown" {
        // The following are required to allow additional functions to be
        // dynamically added and run correctly.
        println!("cargo:rustc-link-arg=--export=__stack_pointer");
        println!("cargo:rustc-link-arg=--export-table");
        println!("cargo:rustc-link-arg=--growable-table");
    }
}
