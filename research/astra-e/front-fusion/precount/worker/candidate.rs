//! Precount virtual run caps and feed the AVX2 run front directly to narrow/ahead.
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

include!("adapter.inc.rs");
