#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::LirInstaller;

#[cfg(not(target_arch = "wasm32"))]
mod cranelift;
#[cfg(not(target_arch = "wasm32"))]
pub use cranelift::LirInstaller;
