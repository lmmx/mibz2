//! Matched copy of the original run front and frozen scalar-cap inverse.
#[allow(dead_code)]
#[path="kernel.rs"] mod kernel;
#[cfg(target_arch="x86_64")]
#[path="stock.front.rs"] mod accelerated;
include!("common.rs");
