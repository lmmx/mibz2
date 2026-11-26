//! Feed true MTF runs directly and elide cap counting under the measured edge admission.
#[allow(dead_code)]
#[path = "kernel.rs"]
mod kernel;

mod front {
    #[derive(Default)]
    pub(super) struct Bits {
        pub(super) lit: Vec<u64>,
        pub(super) one: Vec<u64>,
    }
}

#[cfg(target_arch = "x86_64")]
#[path = "symbols.rs"]
mod pre;

include!("gate.inc.rs");
include!("adapter.inc.rs");
