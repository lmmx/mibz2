//! Preserve the measured edge admissions using a vectorized, mark-independent certificate.
#[allow(dead_code)]
#[path = "../../../narrow-short/constructor_ahead.rs"]
pub mod kernel;

mod frozen {
    include!("../../../edge-budget/gate.rs");

    pub(super) fn marked(records: &[u16], primary: u32) -> bool {
        fits(records, primary)
    }
}

#[allow(dead_code)]
mod envelope {
    include!("geometry.rs");
    include!("simd.rs");
}

fn fits(records: &[u16], primary: u32) -> bool {
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        // The runtime feature check enables the AVX2 loads. Both overlapping windows are
        // wholly in the records slice and the tail is handled scalarly.
        let (rows, strips) = unsafe { envelope::unmarked_avx2(records) };
        if envelope::charge(rows + 1, strips + 1) <= 1 << 20 { return true; }
        if envelope::charge(rows, strips) > 1 << 20 { return false; }
    }
    frozen::marked(records, primary)
}

include!("../../../edge-budget/adapter.inc.rs");
