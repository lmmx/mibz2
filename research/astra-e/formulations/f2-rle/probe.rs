use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{runs, Kernels};
#[path = "section.rs"]
mod section;

fn audit(
    caps: &[u16],
    n: usize,
    p: usize,
    h: &section::Hazards,
) -> (usize, usize, usize, usize, usize) {
    let ll: Vec<_> = caps
        .iter()
        .flat_map(|&v| std::iter::repeat_n((v >> 8) as u8, (v & 255) as usize))
        .collect();
    let mut psi: Vec<_> = (0..n).collect();
    psi.sort_by_key(|&x| ll[x]);
    let mut x = p;
    let mut marking = vec![false; n];
    let mut starts = vec![false; n];
    for &r in &h.events {
        starts[r as usize] = true;
    }
    for &r in &h.danger {
        marking[r as usize] = true;
    }
    let mut text = Vec::with_capacity(n + 3);
    let mut hits = Vec::with_capacity(n);
    let mut period = None;
    for i in 0..n + 3 {
        if i > 0 && x == p {
            period.get_or_insert(i);
        }
        if i < n {
            hits.push((starts[x], marking[x]));
        }
        x = psi[x];
        text.push(ll[x]);
    }
    let (mut actual, mut escape, mut j) = (0, 0, 0);
    for i in 0..n {
        let q = text[i..i + 4].iter().all(|&v| v == text[i]);
        assert_eq!(q, hits[i].0);
        actual += usize::from(q);
    }
    text.truncate(n);
    while j + 4 <= n {
        if text[j..j + 4].iter().all(|&b| b == text[j]) {
            assert!(hits[j..(j + 5).min(n)].iter().all(|z| z.1));
            escape += 1;
            j += 5;
        } else {
            j += 1;
        }
    }
    let mut direct = Vec::new();
    let mut expected = Vec::new();
    let mut outcome = Ok(());
    let k = Kernels::scalar();
    let (mut i, mut protected, mut chunks) = (0, 0, 0);
    while i < n {
        let mut stop = i + 1;
        while stop < n && hits[i].1 == hits[stop].1 {
            stop += 1;
        }
        if hits[i].1 {
            protected += stop - i;
            chunks += 1;
            outcome = mibz2::rle::unrle(&text[i..stop], &mut direct, k);
            if outcome.is_err() {
                assert_eq!(stop, n);
                break;
            }
        } else {
            direct.extend_from_slice(&text[i..stop]);
        }
        i = stop;
    }
    assert_eq!(outcome, mibz2::rle::unrle(&text, &mut expected, k));
    if outcome.is_ok() {
        assert_eq!(direct, expected);
    }
    (period.unwrap(), actual, escape, protected, chunks)
}
fn main() {
    let mut h = section::Hazards::new();
    if std::env::args().nth(1).as_deref() == Some("small") {
        let mut cases = 0;
        for n in 1..=10 {
            for mask in 0..1 << n {
                let caps: Vec<_> = (0..n)
                    .map(|i| if mask & (1 << i) == 0 { 1 } else { 0xff01 })
                    .collect();
                assert!(h.build(&caps, n));
                for p in 0..n {
                    audit(&caps, n, p, &h);
                    cases += 1;
                }
            }
        }
        println!("{cases} arbitrary marked columns, including truncated runs, overlapped quartets and multiple LF cycles");
        return;
    }
    let data = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan = Scanner::new(SliceSource(&data));
    let mut r = runs::Runs::default();
    let mut ordinal = 0;
    println!("stream,n,admitted,run_caps,marked_period,quartet_ranks,marked_quartet_visits,actual_escapes,danger_ranks,sealed_ranks,marked_danger_bytes,marked_danger_spans,extra_retained_rows,rank_probes,select_queries,diagnostic_index_capacity");
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let n = runs::front(&symbols, &mut r, Kernels::detect()).unwrap();
        if n < 65536 {
            continue;
        }
        let selected = ordinal % 24 == 0;
        ordinal += 1;
        if !selected {
            continue;
        }
        let caps = r.records();
        let p = symbols.orig_ptr;
        assert!(h.build(caps, n));
        let (period, actual, escapes, protected, chunks) = audit(caps, n, p as usize, &h);
        let mut extra = 0usize;
        let mut j = 0;
        let mut pos = 0u32;
        let mut i = 0;
        while i < caps.len() {
            let c = caps[i] >> 8;
            let mut end = pos;
            while i < caps.len() && caps[i] >> 8 == c {
                end += u32::from(caps[i] & 255);
                i += 1;
            }
            while j < h.seal.len() && h.seal[j] < end {
                extra += usize::from(end - pos > 16 && h.seal[j] != p);
                j += 1;
            }
            pos = end;
        }
        println!("{index},{n},{},{},{period},{},{actual},{escapes},{},{},{protected},{chunks},{extra},{},{},{}",caps.len()*20<n*3,caps.len(),h.events.len(),h.danger.len(),h.seal.len(),h.probes,h.select,h.index_bytes());
    }
}
