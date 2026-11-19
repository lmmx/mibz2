use mibz2::{
    bwt::LazyCells,
    decode_block, decode_byte_block,
    stream::{Event, Scanner, SliceSource},
    Kernels, Scratch,
};
use std::path::Path;

fn check(path: &Path, oracle: bool, k: Kernels) {
    let data = std::fs::read(path).unwrap();
    let (mut a, mut b) = (
        Scanner::new(SliceSource(&data)),
        Scanner::new(SliceSource(&data)),
    );
    let (mut x, mut y) = (Vec::new(), Vec::new());
    let (mut old, mut new) = (
        Scratch::<LazyCells>::default(),
        Scratch::<LazyCells>::default(),
    );
    let (mut wide, mut narrow, mut odd, mut total, mut eob_max) = (0, 0, 0, 0, 0);
    let expected = oracle.then(|| std::fs::read(path.with_extension("raw")).unwrap());
    let mut outcome = "ok".to_owned();
    loop {
        match (a.next(x), b.next_width::<true>(y)) {
            (Ok(Event::Done), Ok(Event::Done)) => break,
            (Err(e), Err(q)) => {
                assert_eq!(e, q);
                outcome = format!("scan_{:?}", e.kind);
                break;
            }
            (
                Ok(Event::Block {
                    index,
                    bit,
                    end,
                    stream,
                    level,
                    symbols: l,
                }),
                Ok(Event::Block {
                    index: i,
                    bit: j,
                    end: z,
                    stream: s,
                    level: v,
                    symbols: r,
                }),
            ) => {
                assert_eq!((index, bit, end, stream, level), (i, j, z, s, v));
                assert_eq!(
                    (l.crc, l.orig_ptr, l.eob, l.max_len, l.seq_to_unseq),
                    (r.crc, r.orig_ptr, r.eob, r.max_len, r.seq_to_unseq)
                );
                eob_max = eob_max.max(l.eob);
                if let Some(bytes) = r.byte_tape() {
                    assert!(l.eob <= 255);
                    assert!(bytes
                        .iter()
                        .copied()
                        .map(u16::from)
                        .eq(l.syms.iter().copied()));
                    narrow += 1;
                    odd += bytes.len() & 1;
                } else {
                    assert!(l.eob > 255);
                    assert_eq!(l.syms, r.syms);
                    wide += 1;
                }
                let (mut oa, mut ob) = (Vec::new(), Vec::new());
                let ra = decode_block(&l, k, &mut old, &mut oa);
                let rb = decode_byte_block(&r, k, &mut new, &mut ob);
                assert_eq!(ra, rb);
                assert!(oa == ob);
                if let Err(e) = ra {
                    outcome = format!("block_{e:?}");
                    break;
                }
                if let Some(ref expected) = expected {
                    assert!(ob == expected[total..total + ob.len()]);
                }
                total += ob.len();
                x = l.syms;
                y = r.syms;
            }
            _ => panic!("different scanner decisions on {}", path.display()),
        }
    }
    if let Some(expected) = expected {
        assert_eq!(outcome, "ok");
        assert_eq!(total, expected.len());
    }
    println!(
        "{},{},{wide},{narrow},{odd},{eob_max},{total},{}",
        path.file_name().unwrap().to_string_lossy(),
        if k.avx2() { "detected" } else { "scalar" },
        outcome.replace(',', " ")
    );
}
fn main() {
    println!("case,kernels,wide_blocks,narrow_blocks,odd_narrow,eob_max,output_bytes,outcome");
    let dir = std::env::args_os().nth(1).unwrap();
    if Path::new(&dir).is_file() {
        for k in [Kernels::scalar(), Kernels::detect()] {
            check(Path::new(&dir), false, k);
        }
        return;
    }
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| p.extension().is_some_and(|s| s == "bz2"))
        .collect();
    files.sort();
    for f in files {
        for k in [Kernels::scalar(), Kernels::detect()] {
            check(&f, f.with_extension("raw").is_file(), k);
        }
    }
}
