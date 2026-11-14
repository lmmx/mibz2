//! The memory census's markers (feature `memtrace`, research only; off by default and
//! compiled out without it): valgrind client requests to the out-of-tree `memtrace` tool
//! (docs/journal/2026-11-14-cells-memory/memtrace/). Natively each request is a no-op
//! sequence; under the tool it turns the address trace on or off, or puts a marker into the
//! trace, in order with the accesses around it.
//!
//! Markers: (code, four words).
//! - 1 BEGIN (path, block, n, run records): path 1 Cells, 2 Segmented
//! - 2 PHASE (phase): the accesses that follow belong to it
//! - 3 REGION (region, base address, bytes this block reserves, capacity in bytes)
//! - 4 END (outcome, period, cells, an address on the stack): outcome 0 admitted, else refused
#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

/// Which kernel's attempts the trace records: 0 neither, 1 Cells, 2 Segmented
pub static MODE: AtomicU8 = AtomicU8::new(0);
/// The block the driver is decoding
pub static BLOCK: AtomicU64 = AtomicU64::new(0);

pub const CELLS: u8 = 1;
pub const SEGMENTED: u8 = 2;

pub mod phase {
    pub const SLOTS_COUNT: usize = 10;
    pub const SLOTS_ALLOC: usize = 11;
    pub const SLOTS_LETTERS: usize = 12;
    pub const INDUCE: usize = 20;
    pub const WRITE_FILL: usize = 30;
    pub const WRITE_PLACE: usize = 31;
    pub const WRITE_SPINE: usize = 32;
    pub const WRITE_DESCEND: usize = 33;
    pub const WRITE_EXPAND: usize = 34;
    pub const WRITE_RESPINE: usize = 35;
    pub const WRITE_REPEAT: usize = 36;
    pub const SEG_FILL: usize = 50;
    pub const SEG_WALK: usize = 51;
    pub const SEG_ASSEMBLE: usize = 52;
}

pub mod region {
    pub const RECS: usize = 1;
    pub const BUF_PAD: usize = 2;
    pub const BUF_TOPS: usize = 3;
    pub const BUF_BOTS: usize = 4;
    pub const BUF_ARENA: usize = 5;
    pub const BUF_RULES: usize = 6;
    pub const BUF_SPARE: usize = 7;
    pub const POS: usize = 8;
    pub const OUT: usize = 9;
    pub const TT: usize = 20;
    pub const SEG_STARTS: usize = 21;
    pub const SEG_BITMAP: usize = 22;
    pub const SEG_NEXT: usize = 23;
    pub const SEG_BUFS: usize = 24;
    pub const SEG_BUFS_OUTER: usize = 25;
    pub const FALLBACK_TT: usize = 40;
    pub const FALLBACK_SEG: usize = 41;
}

#[inline(always)]
fn request(req: usize, a: [usize; 5]) {
    let args = [req, a[0], a[1], a[2], a[3], a[4]];
    #[cfg(target_arch = "x86_64")]
    // SAFETY: valgrind's client-request preamble (valgrind.h, amd64-linux): rdi is rotated
    // by 128 bits in all (unchanged), `xchg rbx, rbx` changes nothing, and natively rdx keeps
    // its value. Under valgrind the request reads `args` and sets rdx.
    unsafe {
        let mut r: usize = 0;
        core::arch::asm!(
            "rol rdi, 3",
            "rol rdi, 13",
            "rol rdi, 61",
            "rol rdi, 51",
            "xchg rbx, rbx",
            inout("rdx") r,
            in("rax") args.as_ptr(),
            inout("rdi") 0usize => _,
        );
        let _ = r;
    }
    let _ = args;
}

const BASE: usize = (b'M' as usize) << 24 | (b'T' as usize) << 16;

#[inline(always)]
pub fn active(path: u8) -> bool {
    MODE.load(Ordering::Relaxed) == path
}
#[inline(always)]
pub fn on() {
    request(BASE, [0; 5]);
}
#[inline(always)]
pub fn off() {
    request(BASE + 1, [0; 5]);
}
#[inline(always)]
pub fn mark(code: usize, a: usize, b: usize, c: usize, d: usize) {
    request(BASE + 2, [code, a, b, c, d]);
}
#[inline(always)]
pub fn begin(path: u8, n: usize, records: usize) {
    mark(
        1,
        path as usize,
        BLOCK.load(Ordering::Relaxed) as usize,
        n,
        records,
    );
    on();
}
#[inline(always)]
pub fn phase(p: usize) {
    mark(2, p, 0, 0, 0);
}
#[inline(always)]
pub fn region<T>(id: usize, base: *const T, reserved: usize, cap: usize) {
    mark(3, id, base as usize, reserved, cap);
}
#[inline(always)]
pub fn end(outcome: usize, period: usize, cells: usize) {
    let here = 0u8;
    mark(
        4,
        outcome,
        period,
        cells,
        std::hint::black_box(&here) as *const u8 as usize,
    );
}
