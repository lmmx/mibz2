//! Saturate the protected byte-cylinder cover by the existing Open return partition.
#[allow(dead_code)]
#[path = "open.rs"]
mod kernel;
#[allow(dead_code)]
#[path = "../diagonal/section.rs"]
mod section;
include!("adapter.inc.rs");
