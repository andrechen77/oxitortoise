//! The interpreter that executes code in the engine. This module also includes
//! the internal runtime representation of NetLogo code. Interpretation of code
//! is not performance sensitive, so the interpreter is not optimized for speed.
//! However, the internal representation should be conducive to JIT compilation,
//! which is performance sensitive.

mod ir;
