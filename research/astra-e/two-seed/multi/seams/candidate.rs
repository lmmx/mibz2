//! Four positive arcs through the expanded-output hook, with certified sliced RLE1.
const SLICED:bool=true;
#[allow(dead_code)]
#[path="target/generated/expanded.rs"]
mod kernel;
include!("target/generated/gate.rs");
include!("target/generated/expanded-adapter.rs");
