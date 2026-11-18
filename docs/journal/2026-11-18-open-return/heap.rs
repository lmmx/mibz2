//! Peak additional requested heap on one fresh, complete decoder call.
use mibz2::{
    bwt::{Cells, LazyCells},
    open_return::{switches::Switch, Open},
    Kernels,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
struct Ledger;
fn add(bytes: usize) {
    let now = LIVE.fetch_add(bytes, Relaxed) + bytes;
    PEAK.fetch_max(now, Relaxed);
}
unsafe impl GlobalAlloc for Ledger {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(l) };
        if !ptr.is_null() {
            add(l.size());
        }
        ptr
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(l) };
        if !ptr.is_null() {
            add(l.size());
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, l: Layout) {
        unsafe { System.dealloc(ptr, l) };
        LIVE.fetch_sub(l.size(), Relaxed);
    }
    unsafe fn realloc(&self, ptr: *mut u8, old: Layout, new: usize) -> *mut u8 {
        let result = unsafe { System.realloc(ptr, old, new) };
        if !result.is_null() {
            if new > old.size() {
                add(new - old.size());
            } else {
                LIVE.fetch_sub(old.size() - new, Relaxed);
            }
        }
        result
    }
}
#[global_allocator]
static ALLOC: Ledger = Ledger;
fn main() {
    let file = std::env::args().nth(1).expect("INPUT.bz2");
    let input = std::fs::read(file).unwrap();
    let k = Kernels::detect();
    let oracle = mibz2::decode(&input).unwrap();
    println!("arm,peak_requested,returned_length");
    for arm in [
        "cells", "lazy", "16", "s16", "i4", "i8", "i16", "i32", "q16", "qi16", "ranked",
    ] {
        let pre = LIVE.load(Relaxed);
        PEAK.store(pre, Relaxed);
        let out = match arm {
            "cells" => mibz2::decode_with::<Cells>(&input, k),
            "lazy" => mibz2::decode_with::<LazyCells>(&input, k),
            "16" => mibz2::decode_with::<Open<16>>(&input, k),
            "s16" => mibz2::decode_with::<Switch<16>>(&input, k),
            "i4" => mibz2::decode_with::<Open<4, true>>(&input, k),
            "i8" => mibz2::decode_with::<Open<8, true>>(&input, k),
            "i16" => mibz2::decode_with::<Open<16, true>>(&input, k),
            "i32" => mibz2::decode_with::<Open<32, true>>(&input, k),
            "q16" => mibz2::decode_with::<Open<16, false, true>>(&input, k),
            "qi16" => mibz2::decode_with::<Open<16, true, true>>(&input, k),
            "ranked" => mibz2::decode_with::<Open<16, true, true, true>>(&input, k),
            _ => unreachable!(),
        }
        .unwrap();
        let peak = PEAK.load(Relaxed) - pre;
        assert_eq!(out, oracle);
        println!("{arm},{peak},{}", out.len());
    }
}
