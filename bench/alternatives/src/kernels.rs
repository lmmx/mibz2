//! mibz2's inverse BWT against bz2ngrs's (src/ngrs_bwt.rs, its src/bwt.rs as it is) and,
//! built with LIBSAIS_DIR (bench/libsais.sh), libsais 2.12.1's, on the same blocks, on one
//! thread: each kernel's build, walk and output assembly timed apart where it has them.
//!
//! ```text
//! kernels FILE.bz2 [PASSES] [OUT.jsonl]
//! ```
//!
//! The file's blocks are read once, each to its last column (mibz2's scanner and MTF, whose
//! byte counts mibz2's build takes and bz2ngrs's counts again). An untimed pass then
//! checks that both kernels give every block the same bytes, and that they RLE1-decode to
//! its CRC. Then each timed pass runs both kernels over every block in decode order, the
//! kernels' order alternating pass to pass. A block's last column is copied into the
//! kernel's own input buffer, untimed, before its build, as a decoder's MTF writes it.
//!
//! - mibz2: `build` (an init pass over the column, then the scatter, counts from MTF);
//!   `walk_segments_room_parts` (its starts chosen, the lanes walked into segment
//!   buffers); `SegScratch::assemble` (the segments joined into the block).
//! - bz2ngrs: `build_t` (its 8-part count, then the scatter); `mark_splitters` and `walk`
//!   (the lanes walked into 1 KiB chunks in the column's own buffer); `gather` (the chunks
//!   copied into the block in text order). Its count alone is also timed (`count_only`).
//! - libsais (`cfg(mibz2_libsais)`): `libsais_unbwt_cyclic_ctx` on a context made once, the
//!   counts from MTF as mibz2's build takes them, one call a block: its build (the bigram
//!   table), walk and joining are inside it, so only its total is timed. It is an inverse
//!   BWT, not a bzip2 decoder.
//!
//! The kernels' order rotates a step each pass and reverses every other cycle. Each pass is
//! one JSON line a kernel (null for a phase it doesn't time apart). The summary is each
//! phase's median over passes in ns per byte, and each other kernel's over mibz2's, paired
//! by pass, with a 95% t-interval.

// (bz2ngrs's file as it is, its own formatting kept)
#[rustfmt::skip]
mod ngrs_bwt;

use std::io::Write;
use std::time::Instant;

use mibz2::bwt::{build, walk_segments_room_parts, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

struct Block {
    ll: Vec<u8>,
    counts: [u32; 256],
    orig_ptr: u32,
    crc: u32,
}

fn blocks(data: &[u8], k: Kernels) -> Vec<Block> {
    let mut sc = Scanner::new(SliceSource(data));
    let mut v = Vec::new();
    let mut syms = Vec::new();
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        v.push(Block {
            ll,
            counts,
            orig_ptr: b.orig_ptr,
            crc: b.crc,
        });
        syms = b.syms;
    }
    v
}

struct Mibz2 {
    ll: Vec<u8>,
    tt: TVector,
    seg: SegScratch,
    walked: Vec<u8>,
}

struct Ngrs {
    bwt: ngrs_bwt::Bwt,
    ll: Box<[u8; ngrs_bwt::BYTES]>,
    text: Box<[u8; ngrs_bwt::BYTES]>,
}

#[cfg(mibz2_libsais)]
extern "C" {
    fn libsais_unbwt_create_ctx() -> *mut std::ffi::c_void;
    fn libsais_unbwt_cyclic_ctx(
        ctx: *const std::ffi::c_void,
        t: *const u8,
        u: *mut u8,
        a: *mut i32,
        n: i32,
        freq: *const i32,
        i: i32,
    ) -> i32;
}

/// libsais's context and buffers, kept from block to block: its output, its 4-byte-a-row
/// table and the counts
#[cfg(mibz2_libsais)]
struct Sais {
    ctx: *mut std::ffi::c_void,
    u: Vec<u8>,
    a: Vec<i32>,
    freq: Vec<i32>,
}

/// One block through libsais's kernel: its total in seconds
#[cfg(mibz2_libsais)]
fn run_sais(x: &mut Sais, b: &Block) -> f64 {
    let n = b.ll.len();
    x.freq.clear();
    x.freq.extend(b.counts.iter().map(|&c| c as i32));
    x.u.resize(n, 0);
    x.a.resize(n + 1, 0);
    let t0 = Instant::now();
    // SAFETY: the context is libsais's own, made once and never freed while used; `u` holds n
    // bytes and `a` n + 1 words, as libsais_unbwt_cyclic_ctx takes them; `ll` holds n bytes,
    // `freq` 256 counts of them, and orig_ptr < n (the scanner checks it)
    let r = unsafe {
        libsais_unbwt_cyclic_ctx(
            x.ctx,
            b.ll.as_ptr(),
            x.u.as_mut_ptr(),
            x.a.as_mut_ptr(),
            n as i32,
            x.freq.as_ptr(),
            b.orig_ptr as i32,
        )
    };
    let t = t0.elapsed().as_secs_f64();
    assert_eq!(r, 0, "libsais_unbwt_cyclic_ctx");
    t
}

/// One block through mibz2's kernel: [build, walk, assemble] in seconds
fn run_mibz2(m: &mut Mibz2, b: &Block) -> [f64; 3] {
    m.ll.clear();
    m.ll.extend_from_slice(&b.ll);
    let t0 = Instant::now();
    build(&m.ll, &b.counts, &mut m.tt);
    let t1 = Instant::now();
    let segmented = walk_segments_room_parts(&mut m.tt, b.orig_ptr, &mut m.walked, &mut m.seg);
    let t2 = Instant::now();
    if segmented {
        m.seg.assemble(&mut m.walked);
    }
    let t3 = Instant::now();
    [
        (t1 - t0).as_secs_f64(),
        (t2 - t1).as_secs_f64(),
        (t3 - t2).as_secs_f64(),
    ]
}

/// One block through bz2ngrs's kernel: [build, walk, gather] in seconds
fn run_ngrs(g: &mut Ngrs, b: &Block) -> [f64; 3] {
    g.ll[..b.ll.len()].copy_from_slice(&b.ll);
    ngrs_bwt::inverse_timed(
        &mut g.bwt,
        &mut g.ll,
        b.ll.len() as u32,
        b.orig_ptr,
        &mut g.text,
    )
}

fn zeroed() -> Box<[u8; ngrs_bwt::BYTES]> {
    vec![0u8; ngrs_bwt::BYTES]
        .into_boxed_slice()
        .try_into()
        .unwrap()
}

/// The kernels' order in pass `p` of `n`: rotated a step each pass, reversed every other
/// cycle, so that over 2n passes each kernel runs in each position equally often
fn order(p: usize, n: usize) -> Vec<usize> {
    let mut o: Vec<usize> = (0..n).collect();
    o.rotate_left(p % n);
    if (p / n) % 2 == 1 {
        o.reverse();
    }
    o
}

fn t975(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    T.get(df.wrapping_sub(1)).copied().unwrap_or(1.96)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// The geometric mean of the ratios, with its 95% t-interval
fn ratio(r: &[f64]) -> (f64, f64, f64) {
    let lg: Vec<f64> = r.iter().map(|x| x.ln()).collect();
    let n = lg.len();
    let m = lg.iter().sum::<f64>() / n as f64;
    if n < 2 {
        return (m.exp(), f64::NAN, f64::NAN);
    }
    let sd = (lg.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt();
    let h = t975(n - 1) * sd / (n as f64).sqrt();
    (m.exp(), (m - h).exp(), (m + h).exp())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("kernels FILE.bz2 [PASSES] [OUT.jsonl]");
    let passes: usize = args.get(2).map_or(10, |s| s.parse().unwrap());
    let mut out = args.get(3).map(|p| std::fs::File::create(p).unwrap());
    let k = Kernels::detect();
    let data = std::fs::read(path).unwrap();
    let bs = blocks(&data, k);
    let bytes: usize = bs.iter().map(|b| b.ll.len()).sum();
    assert!(
        bs.iter().all(|b| b.ll.len() <= 900_000),
        "a block over 900,000 bytes"
    );

    let mut m = Mibz2 {
        ll: Vec::new(),
        tt: TVector::new(),
        seg: SegScratch::default(),
        walked: Vec::new(),
    };
    let mut g = Ngrs {
        bwt: ngrs_bwt::Bwt::new(),
        ll: zeroed(),
        text: zeroed(),
    };
    #[cfg(mibz2_libsais)]
    let mut sais = Sais {
        // SAFETY: no arguments; a null result is checked below
        ctx: unsafe { libsais_unbwt_create_ctx() },
        u: Vec::new(),
        a: Vec::new(),
        freq: Vec::new(),
    };
    #[cfg(mibz2_libsais)]
    assert!(!sais.ctx.is_null(), "libsais_unbwt_create_ctx");
    let mut names = vec!["mibz2", "bz2ngrs"];
    if cfg!(mibz2_libsais) {
        names.push("libsais");
    }

    // the check: the same bytes from every kernel, and each block's CRC after RLE1
    let mut rled = Vec::new();
    for (i, b) in bs.iter().enumerate() {
        run_mibz2(&mut m, b);
        run_ngrs(&mut g, b);
        let n = b.ll.len();
        assert_eq!(m.walked.len(), n, "block {i}: mibz2's length");
        assert!(
            m.walked == g.text[..n],
            "block {i}: bz2ngrs's bytes differ from mibz2's"
        );
        #[cfg(mibz2_libsais)]
        {
            run_sais(&mut sais, b);
            assert!(
                m.walked == sais.u,
                "block {i}: libsais's bytes differ from mibz2's"
            );
        }
        rled.clear();
        rle::unrle(&m.walked, &mut rled, k).unwrap();
        assert_eq!(crc::crc32(&rled, k), b.crc, "block {i}: the CRC");
    }
    eprintln!(
        "checked: {} blocks, {bytes} bytes, the same from every kernel ({}), every CRC",
        bs.len(),
        names.join(", ")
    );

    // [kernel][phase] per pass, phase 3 the total (a phase a kernel doesn't time apart is
    // NaN); and bz2ngrs's count alone
    let mut t: Vec<[Vec<f64>; 4]> = names.iter().map(|_| Default::default()).collect();
    let mut counts = Vec::new();
    for pass in 0..passes {
        for kn in order(pass, names.len()) {
            let mut s = [0f64; 3];
            let mut tot = 0.0;
            for b in &bs {
                let x = match kn {
                    0 => run_mibz2(&mut m, b),
                    1 => run_ngrs(&mut g, b),
                    #[cfg(mibz2_libsais)]
                    _ => {
                        tot += run_sais(&mut sais, b);
                        continue;
                    }
                    #[cfg(not(mibz2_libsais))]
                    _ => unreachable!(),
                };
                for j in 0..3 {
                    s[j] += x[j];
                }
                tot += x.iter().sum::<f64>();
            }
            let phases = kn < 2;
            let cell = |v: f64| {
                if phases {
                    format!("{v:.6}")
                } else {
                    "null".into()
                }
            };
            for j in 0..3 {
                t[kn][j].push(if phases { s[j] } else { f64::NAN });
            }
            t[kn][3].push(tot);
            if let Some(f) = out.as_mut() {
                writeln!(
                    f,
                    "{{\"pass\":{pass},\"kernel\":\"{}\",\"bytes\":{bytes},\"build_s\":{},\"walk_s\":{},\"assemble_s\":{},\"total_s\":{tot:.6}}}",
                    names[kn],
                    cell(s[0]),
                    cell(s[1]),
                    cell(s[2])
                )
                .unwrap();
            }
        }
        let (mut z, mut c) = (0, 0.0);
        for b in &bs {
            g.ll[..b.ll.len()].copy_from_slice(&b.ll);
            let t0 = Instant::now();
            z += ngrs_bwt::count_only(&g.ll, b.ll.len() as u32);
            c += t0.elapsed().as_secs_f64();
        }
        std::hint::black_box(z);
        counts.push(c);
    }
    let ns = |s: f64| s * 1e9 / bytes as f64;
    println!(
        "{} blocks, {:.1} MB of last columns, {passes} passes, one thread\n",
        bs.len(),
        bytes as f64 / 1e6
    );
    let others = &names[1..];
    println!(
        "| phase | {} | {} |",
        names
            .iter()
            .map(|n| format!("{n} ns/B"))
            .collect::<Vec<_>>()
            .join(" | "),
        others
            .iter()
            .map(|n| format!("{n} / mibz2, paired: mean (95%)"))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    println!("|---|{}", "--:|".repeat(names.len() + others.len()));
    let phase = ["build", "walk", "assemble / gather", "inverse BWT, total"];
    for j in 0..4 {
        let mut cells: Vec<String> = t
            .iter()
            .map(|k| {
                let v = median(k[j].clone());
                if v.is_nan() {
                    "(in its total)".into()
                } else {
                    format!("{:.3}", ns(v))
                }
            })
            .collect();
        for kn in 1..names.len() {
            let r: Vec<f64> = (0..passes).map(|p| t[kn][j][p] / t[0][j][p]).collect();
            if r.iter().any(|v| v.is_nan()) {
                cells.push("".into());
            } else {
                let (gm, lo, hi) = ratio(&r);
                cells.push(format!("{gm:.3} ({lo:.3}-{hi:.3})"));
            }
        }
        println!("| {} | {} |", phase[j], cells.join(" | "));
    }
    println!(
        "\nbz2ngrs's count alone (its 8-part histogram, in its build above): {:.3} ns/B",
        ns(median(counts))
    );
}
