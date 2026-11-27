//! Peak additional requested heap on one fresh, complete decoder call.
use mibz2::{
    bwt::{Cells, LazyCells, Segmented},
    extrema::Extrema,
    valleys::Valleys,
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
    for arm in ["cells", "lazy", "segmented", "valley", "extrema", "direct"] {
        let pre = LIVE.load(Relaxed);
        PEAK.store(pre, Relaxed);
        let out = match arm {
            "cells" => mibz2::decode_with::<Cells>(&input, k),
            "lazy" => mibz2::decode_with::<LazyCells>(&input, k),
            "segmented" => mibz2::decode_with::<Segmented>(&input, k),
            "valley" => mibz2::decode_with::<Valleys<false>>(&input, k),
            "extrema" => mibz2::decode_with::<Extrema>(&input, k),
            "direct" => mibz2::extrema::direct(&input, k),
            _ => unreachable!(),
        }
        .unwrap();
        let peak = PEAK.load(Relaxed) - pre;
        assert_eq!(out, oracle);
        println!("{arm},{peak},{}", out.len());
    }
}
