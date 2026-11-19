// Included in lib.rs of the disposable source tree.
#[repr(align(64))]
struct Mark([std::sync::atomic::AtomicU8; 64]);
#[no_mangle]
static ASTRA_MARK: Mark = Mark([const { std::sync::atomic::AtomicU8::new(0) }; 64]);

#[inline(always)]
fn astra_mark(i: usize) {
    use std::sync::atomic::{compiler_fence, Ordering};
    compiler_fence(Ordering::SeqCst);
    ASTRA_MARK.0[i].store(1, Ordering::Relaxed);
    compiler_fence(Ordering::SeqCst);
}

fn region(name: &str, address: usize, bytes: usize) {
    if bytes > 0 { println!("range {name} {address:x} {bytes}"); }
}
fn vector<T>(name: &str, v: &Vec<T>) {
    region(name, v.as_ptr() as usize, v.capacity() * std::mem::size_of::<T>());
}

// One real scanner block and the worker entry, with the worker's recycled capacities.
// Scope markers bracket only the second decode. Diagnostics run outside that interval.
pub fn astra_block(path: &str, out_path: &str, inverse_path: &str) {
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();
    assert!(k.avx2 && k.clmul);
    let mut scratch = Scratch::<bwt::LazyCells>::default();
    let mut syms = Vec::new();
    let mut output = Vec::new();
    for lap in 0..2 {
        let mut sc = stream::Scanner::new(stream::SliceSource(&data));
        output.clear();
        if lap == 1 { astra_mark(31); }
        astra_mark(1);
        let stream::Event::Block { symbols, .. } = sc.next(syms).unwrap() else { panic!() };
        astra_mark(2);
        decode_block(&symbols, k, &mut scratch, &mut output).unwrap();
        astra_mark(11);
        syms = symbols.syms;
        assert!(matches!(sc.next(Vec::new()).unwrap(), stream::Event::Done));
        if lap == 1 { astra_mark(32); }
    }
    vector("compressed", &data);
    vector("symbols", &syms);
    vector("inverse", &scratch.ll);
    vector("rle", &scratch.rle);
    vector("final", &output);
    region("scratch_counts", scratch.counts.as_ptr() as usize, 1024);
    scratch.runs.astra_ranges();
    scratch.inverse.astra_ranges(scratch.ll.as_ptr() as usize);
    println!("info output {} rows {} compressed {} symbols {}", output.len(), scratch.ll.len(), data.len(), syms.len());
    std::fs::write(out_path, output).unwrap();
    std::fs::write(inverse_path, &scratch.ll).unwrap();
}
