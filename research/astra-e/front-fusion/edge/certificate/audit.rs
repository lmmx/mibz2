use mibz2::{Kernels, runs, stream::{Event, Scanner, SliceSource}};
include!("geometry.rs");
include!("simd.rs");

fn bitmaps(s: &[u16]) -> (Vec<u64>, Vec<u64>) {
    let mut literal = vec![0; s.len() / 64 + 3];
    let mut one = literal.clone();
    for (i, &v) in s.iter().enumerate() {
        if v > 1 { literal[i / 64] |= 1 << (i % 64); }
        if v != 0 { one[i / 64] |= 1 << (i % 64); }
    }
    (literal, one)
}

fn small() {
    let mut count = 0;
    for offset in 0..64 {
        for t in 0..=12 {
            for value in 0..1usize << t {
                let mut syms = vec![2; offset];
                syms.extend((0..t).map(|i| ((value >> i) & 1) as u16));
                let (l, o) = bitmaps(&syms);
                let width = (1 << t) + value - usize::from(offset == 0);
                let rows = offset.saturating_sub(1) + if width <= 16 { width } else { 0 };
                assert_eq!(unmarked(&l, &o, syms.len()), Some((rows, usize::from(width > 16))));
                count += 1;
            }
        }
    }
    println!("{count} checks over every bit alignment and every RUNA/B word of up to 12 symbols");
}

fn main() {
    let input = std::env::args().nth(1).unwrap();
    if input == "small" { small(); return; }
    let data = std::fs::read(input).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut runs = runs::Runs::default();
    println!("stream,n,caps,bitmap_words,unmarked_rows,unmarked_thick,marked_rows,marked_thick,lower,upper,exact");
    while let Ok(Event::Block { index, symbols: s, .. }) = scanner.next(Vec::new()) {
        if !runs::takes_runs(&s) { continue; }
        let n = runs::front(&s, &mut runs, Kernels::detect()).unwrap();
        let end = s.syms.iter().position(|&v| v == s.eob && v > 1).unwrap();
        let (lit, one) = bitmaps(&s.syms[..end]);
        let (rows, thick) = unmarked(&lit, &one, end).unwrap();
        assert_eq!((rows, thick), unmarked_caps(runs.records()));
        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") { assert_eq!((rows,thick),unsafe{unmarked_avx2(runs.records())}); }
        let (mut m, mut h, mut pos, mut at) = (0, 0, 0, 0);
        let caps = runs.records();
        while at < caps.len() {
            let byte = caps[at] >> 8;
            let mut width = 0;
            while at < caps.len() && caps[at] >> 8 == byte {
                width += usize::from(caps[at] & 255); at += 1;
            }
            if width <= 16 { m += width; }
            else if (pos..pos + width).contains(&(s.orig_ptr as usize)) {
                let p = s.orig_ptr as usize;
                m += 1; h += usize::from(p > pos) + usize::from(p + 1 < pos + width);
            } else { h += 1; }
            pos += width;
        }
        assert_eq!(pos, n);
        assert!((rows..=rows+1).contains(&m) && (thick..=thick+1).contains(&h));
        let lower = charge(rows, thick);
        let upper = charge(rows + 1, thick + 1);
        let exact = charge(m, h);
        assert!((lower..=upper).contains(&exact));
        println!("{index},{n},{},{},{rows},{thick},{m},{h},{lower},{upper},{exact}", caps.len(),end/64+1);
    }
}
