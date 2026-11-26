//! Four positive arcs through the expanded-output hook, with the joined contiguous RLE1 control.
const SLICED:bool=false;
#[allow(dead_code)]
#[path="target/generated/expanded.rs"]
mod kernel;
include!("target/generated/gate.rs");
include!("target/generated/expanded-adapter.rs");
