//! Two-bank composed MTF front with the frozen scalar-cap inverse.
#[allow(dead_code)]
#[path="kernel.rs"] mod kernel;
#[cfg(target_arch="x86_64")]
#[path="wide.front.rs"] mod accelerated;
include!("common.rs");
