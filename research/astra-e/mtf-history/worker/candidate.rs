//! Last-occurrence bit selection and right-aligned MTF tail on the verified scalar-cap inverse.
#[allow(dead_code)]
#[path="../../mtf-pairs/worker/kernel.rs"] mod kernel;
#[cfg(target_arch="x86_64")]
#[path="history.front.rs"] mod accelerated;
include!("../../mtf-pairs/worker/common.rs");
