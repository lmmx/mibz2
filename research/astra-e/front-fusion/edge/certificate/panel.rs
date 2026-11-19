use std::{cell::Cell, hint::black_box};
thread_local! { static GATE_NS: Cell<u64> = const { Cell::new(0) }; }

fn fits(records: &[u16], mark: u32) -> bool {
    let now = cpu();
    let answer = control(black_box(records), black_box(mark));
    GATE_NS.with(|sum| sum.set(sum.get() + cpu() - now));
    answer
}

struct Input { records: Vec<u16>, primary: u32, lit: Vec<u64>, one: Vec<u64>, end: usize }
impl Input {
    fn check(&self, which: usize) -> bool {
        let records = black_box(self.records.as_slice());
        let mark = black_box(self.primary);
        // Arms 0 and 1 enter the identical noinline frozen function.
        if which < 2 { return control(records, mark); }
        let (rows, thick) = if which == 2 {
            unsafe { unmarked_avx2(records) }
        } else {
            unmarked(black_box(&self.lit),black_box(&self.one),black_box(self.end)).unwrap()
        };
        if charge(rows + 1, thick + 1) <= 1 << 20 { true }
        else if charge(rows, thick) > 1 << 20 { false }
        else { control(records, mark) }
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();
    assert!(is_x86_feature_detected!("avx2"));
    let mut dense = Vec::new();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut records = runs::Runs::default();
    while let Ok(Event::Block { symbols: s, .. }) = sc.next(Vec::new()) {
        if !runs::takes_runs(&s) { continue; }
        let n = runs::front(&s, &mut records, k).unwrap();
        if n < 65_536 || records.records().len() * 20 < 3*n { continue; }
        let end = s.syms.iter().position(|&v| v == s.eob && v > 1).unwrap();
        let mut lit = vec![0; end/64+3];
        let mut one = lit.clone();
        for (i, &v) in s.syms[..end].iter().enumerate() {
            if v > 1 {lit[i/64] |= 1 << (i%64);}
            if v != 0 {one[i/64] |= 1 << (i%64);}
        }
        dense.push(Input{records:records.records().to_vec(),primary:s.orig_ptr,lit,one,end});
    }
    assert_eq!(dense.len(), 5);
    for block in &dense { for arm in 0..4 { assert!(block.check(arm)); } }
    // Frozen code in context with its real predecessor cache state, observing the five gates.
    for pass in 0..3 {
        let mut sc = Scanner::new(SliceSource(&data));
        let mut worker = Arm::default();
        let mut rows = 0;
        let mut count = 0;
        GATE_NS.with(|sum| sum.set(0));
        let start = cpu();
        while let Ok(Event::Block { symbols: s, .. }) = sc.next(Vec::new()) {
            worker.block::<0>(&s,k,false).unwrap();
            assert_eq!(mibz2::crc::crc32(&worker.decoded,k),s.crc);
            rows += worker.decoded.len();
            count += 1;
        }
        let elapsed = cpu() - start;
        assert_eq!((count,rows),(431,209700196));
        eprintln!("full,{pass},{elapsed},{},{}",GATE_NS.with(Cell::get),rows);
    }
    println!("round,position,arm,cpu_ns,passes");
    for round in 0..32 {
        for position in 0..4 {
            let arm = (round % 4 + if round / 4 % 2 == 0 {position} else {4-position}) % 4;
            let start = cpu();
            for _ in 0..4 {
                for b in &dense { assert!(black_box(b.check(arm))); }
            }
            println!("{round},{position},{arm},{},4",cpu()-start);
        }
    }
}
