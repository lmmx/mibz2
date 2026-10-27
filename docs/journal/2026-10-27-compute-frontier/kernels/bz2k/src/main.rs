//! bz2k: measurements of the bzip2 decode's phases, and of this machine's memory.
//!
//!   bz2k phases FILE.bz2 [slice8]      each phase's cycles (TSC) per output byte, CRCs checked
//!   bz2k cat FILE.bz2 [slice8|fused]   decode to stdout (for `cmp` against lbzip2 -dc)
//!   bz2k walk FILE.bz2 BLOCKS          the inverse BWT's walk, 1 list at a time and K in lockstep
//!   bz2k threads FILE.bz2 N            N threads each decoding the whole file at once
//!   bz2k chase                         a dependent-load chain's cycles per load by working set
//!   bz2k bandwidth                     read and copy bandwidth, 1 to 4 threads
//!
//! TSC cycles: this machine's TSC is constant at 2.1 GHz, so they are wall time.
use bz2k::*;
use std::io::Write;
use std::time::Instant;

fn per(c: u64, n: u64) -> f64 {
    c as f64 / n as f64
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let crc_kind = if args.iter().any(|a| a == "simd") { crc::Kind::Simd } else if args.iter().any(|a| a == "slice8") { crc::Kind::Slice8 } else { crc::Kind::Bytewise };
    match args[1].as_str() {
        "phases" => {
            let data = std::fs::read(&args[2]).unwrap();
            let mut out = Vec::new();
            let t = Instant::now();
            let cy = decode_all(&data, &mut out, false, None, crc_kind);
            let wall = t.elapsed().as_secs_f64();
            let total = cy.huffman + cy.mtf + cy.build + cy.walk + cy.unrle_crc;
            let o = cy.out_bytes;
            println!(
                "{{\"file\":\"{}\",\"crc\":\"{:?}\",\"blocks\":{},\"in_bytes\":{},\"rle_bytes\":{},\"out_bytes\":{},\"wall_s\":{:.3},\
                 \"cy_per_out_byte\":{{\"huffman\":{:.2},\"mtf_rle2\":{:.2},\"build_tt\":{:.2},\"walk\":{:.2},\"unrle_crc\":{:.2},\"total\":{:.2}}},\
                 \"cy_per_in_byte\":{:.1},\"cy_per_rle_byte_walk\":{:.2}}}",
                args[2], crc_kind, cy.blocks, cy.in_bits / 8, cy.rle_bytes, o, wall,
                per(cy.huffman, o), per(cy.mtf, o), per(cy.build, o), per(cy.walk, o), per(cy.unrle_crc, o), per(total, o),
                per(total, cy.in_bits / 8), per(cy.walk, cy.rle_bytes)
            );
        }
        "cat" => {
            let data = std::fs::read(&args[2]).unwrap();
            let fused = args.iter().any(|a| a == "fused");
            let stdout = std::io::stdout();
            let mut w = stdout.lock();
            let t = Instant::now();
            let n = if fused { cat_fused(&data, &mut w) } else { cat_phased(&data, &mut w, crc_kind) };
            eprintln!("decoded {} bytes in {:.3}s ({:.2} cycles/byte at 2.1 GHz)", n, t.elapsed().as_secs_f64(),
                      t.elapsed().as_secs_f64() * 2.1e9 / n as f64);
        }
        "walk" => {
            let data = std::fs::read(&args[2]).unwrap();
            let nb: usize = args[3].parse().unwrap();
            let mut prepared = Vec::new();
            let mut out = Vec::new();
            decode_all(&data, &mut out, false, Some(&mut prepared), crc_kind);
            prepared.truncate(nb - nb % 24);
            let huge = args.iter().any(|a| a == "huge");
            let tts: Vec<Vec<u32>> = prepared
                .iter()
                .map(|p| {
                    let mut tt = Vec::new();
                    build(&p.ll, &p.counts, &mut tt);
                    tt
                })
                .collect();
            assert!(tts.iter().all(|t| links_in_bounds(t)));
            let tts: Vec<&[u32]> = if huge { tts.iter().map(|t| &*huge_copy(t)).collect() } else { tts.iter().map(|t| t.as_slice()).collect() };
            println!("{{\"huge_pages\":{},\"anon_huge_kb\":{}}}", huge, anon_huge_kb());
            let bytes: u64 = tts.iter().map(|t| t.len() as u64).sum();
            // the reference: one list at a time
            let mut refs: Vec<Vec<u8>> = vec![Vec::new(); tts.len()];
            let mut best1 = u64::MAX;
            for _ in 0..3 {
                let t0 = rdtsc();
                for (i, tt) in tts.iter().enumerate() {
                    walk(tt, prepared[i].orig_ptr, &mut refs[i]);
                }
                best1 = best1.min(rdtsc() - t0);
            }
            println!("{{\"k\":1,\"blocks\":{},\"cy_per_byte\":{:.2}}}", tts.len(), per(best1, bytes));
            macro_rules! k {
                ($k:literal) => {{
                    let mut best = u64::MAX;
                    let mut outs: [Vec<u8>; $k] = std::array::from_fn(|_| Vec::new());
                    let mut equal = true;
                    for _ in 0..3 {
                        let t0 = rdtsc();
                        let mut el = 0;
                        for g in (0..tts.len()).step_by($k) {
                            let ts: [&[u32]; $k] = std::array::from_fn(|j| tts[g + j]);
                            let ps: [u32; $k] = std::array::from_fn(|j| prepared[g + j].orig_ptr);
                            walk_k::<$k>(ts, ps, &mut outs);
                            el += rdtsc() - t0 - el; // (time so far)
                            for j in 0..$k {
                                equal &= outs[j] == refs[g + j];
                            }
                        }
                        let _ = el;
                        best = best.min(rdtsc() - t0);
                    }
                    println!("{{\"k\":{},\"blocks\":{},\"cy_per_byte_incl_check\":{:.2},\"speedup\":{:.2},\"equal\":{}}}",
                             $k, tts.len(), per(best, bytes), best1 as f64 / best as f64, equal);
                }};
            }
            k!(2);
            k!(4);
            k!(8);
            // without the equality check inside the timing
            macro_rules! kq {
                ($k:literal) => {{
                    let mut best = u64::MAX;
                    let mut outs: [Vec<u8>; $k] = std::array::from_fn(|_| Vec::new());
                    for _ in 0..3 {
                        let t0 = rdtsc();
                        for g in (0..tts.len()).step_by($k) {
                            let ts: [&[u32]; $k] = std::array::from_fn(|j| tts[g + j]);
                            let ps: [u32; $k] = std::array::from_fn(|j| prepared[g + j].orig_ptr);
                            walk_k_raw::<$k>(ts, ps, &mut outs);
                            std::hint::black_box(&outs);
                        }
                        best = best.min(rdtsc() - t0);
                    }
                    println!("{{\"k\":{},\"timed_alone\":true,\"cy_per_byte\":{:.2},\"speedup\":{:.2}}}",
                             $k, per(best, bytes), best1 as f64 / best as f64);
                }};
            }
            kq!(1);
            kq!(2);
            kq!(3);
            kq!(4);
            kq!(6);
            kq!(8);
        }
        "threads" => {
            let data = std::fs::read(&args[2]).unwrap();
            let n: usize = args[3].parse().unwrap();
            let t = Instant::now();
            let res: Vec<(f64, u64)> = std::thread::scope(|s| {
                let hs: Vec<_> = (0..n)
                    .map(|_| {
                        s.spawn(|| {
                            let mut out = Vec::new();
                            let t = Instant::now();
                            let cy = decode_all(&data, &mut out, false, None, crc_kind);
                            (t.elapsed().as_secs_f64(), cy.out_bytes)
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().unwrap()).collect()
            });
            let wall = t.elapsed().as_secs_f64();
            let bytes: u64 = res.iter().map(|r| r.1).sum();
            let per_thread: Vec<String> = res.iter().map(|r| format!("{:.2}", r.0 * 2.1e9 / r.1 as f64)).collect();
            println!("{{\"threads\":{},\"wall_s\":{:.3},\"aggregate_MB_s\":{:.1},\"per_thread_cy_per_byte\":[{}]}}",
                     n, wall, bytes as f64 / wall / 1e6, per_thread.join(","));
        }
        "walkm" => {
            // each block built, then walked twice: cold (just built, as in a decode) and warm
            let data = std::fs::read(&args[2]).unwrap();
            let nb: usize = args[3].parse().unwrap();
            let mut prepared = Vec::new();
            let mut out = Vec::new();
            decode_all(&data, &mut out, false, Some(&mut prepared), crc_kind);
            prepared.truncate(nb);
            let mut tt = Vec::new();
            let mut o = vec![0u8; 900_000];
            let mut rle = Vec::new();
            let mut cy = [0u64; 6];
            let mut bytes = 0u64;
            for p in &prepared {
                build(&p.ll, &p.counts, &mut tt);
                assert!(links_in_bounds(&tt));
                bytes += tt.len() as u64;
                let t0 = rdtsc();
                walk(&tt, p.orig_ptr, &mut rle);
                cy[0] += rdtsc() - t0; // just built: as in the decode
                let t0 = rdtsc();
                walk(&tt, p.orig_ptr, &mut rle);
                cy[1] += rdtsc() - t0; // again: as warm as it gets
                let t0 = rdtsc();
                walk_multi::<2>(&tt, &mut o);
                cy[2] += rdtsc() - t0;
                let t0 = rdtsc();
                walk_multi::<4>(&tt, &mut o);
                cy[3] += rdtsc() - t0;
                let t0 = rdtsc();
                walk_multi::<8>(&tt, &mut o);
                cy[4] += rdtsc() - t0;
                let t0 = rdtsc();
                walk_multi::<16>(&tt, &mut o);
                cy[5] += rdtsc() - t0;
            }
            println!("{{\"blocks\":{},\"cy_per_byte\":{{\"after_build\":{:.2},\"warm\":{:.2},\"multi2\":{:.2},\"multi4\":{:.2},\"multi8\":{:.2},\"multi16\":{:.2}}}}}",
                     prepared.len(), per(cy[0], bytes), per(cy[1], bytes), per(cy[2], bytes), per(cy[3], bytes), per(cy[4], bytes), per(cy[5], bytes));
        }
        "seg" => {
            // walk_segments against walk, block by block, just after each block's build
            let data = std::fs::read(&args[2]).unwrap();
            let nb: usize = args[3].parse().unwrap();
            let m: usize = args[4].parse().unwrap();
            let mut prepared = Vec::new();
            let mut out = Vec::new();
            decode_all(&data, &mut out, false, Some(&mut prepared), crc_kind);
            prepared.truncate(nb);
            let (mut tt, mut a, mut b) = (Vec::new(), Vec::new(), Vec::new());
            let mut cy = [0u64; 5];
            let mut bytes = 0u64;
            let mut equal = true;
            let mut fallback = 0;
            for p in &prepared {
                build(&p.ll, &p.counts, &mut tt);
                assert!(links_in_bounds(&tt));
                bytes += tt.len() as u64;
                let t0 = rdtsc();
                walk(&tt, p.orig_ptr, &mut a);
                cy[0] += rdtsc() - t0;
                macro_rules! s {
                    ($i:literal, $k:literal) => {{
                        let t0 = rdtsc();
                        let ok = walk_segments::<$k>(&tt, p.orig_ptr, m, &mut b);
                        cy[$i] += rdtsc() - t0;
                        if !ok { fallback += 1 } else { equal &= a == b; }
                    }};
                }
                s!(1, 4);
                s!(2, 8);
                macro_rules! s2 {
                    ($i:literal, $k:literal) => {{
                        let t0 = rdtsc();
                        let ok = walk_segments::<$k>(&tt, p.orig_ptr, m, &mut b);
                        cy[$i] += rdtsc() - t0;
                        if !ok { fallback += 1 } else { equal &= a == b; }
                    }};
                }
                s2!(3, 12);
                s2!(4, 16);
            }
            println!("{{\"blocks\":{},\"segments\":{},\"equal\":{},\"fallbacks\":{},\"cy_per_byte\":{{\"walk\":{:.2},\"seg4\":{:.2},\"seg8\":{:.2},\"seg12\":{:.2},\"seg16\":{:.2}}}}}",
                     prepared.len(), m, equal, fallback, per(cy[0], bytes), per(cy[1], bytes), per(cy[2], bytes), per(cy[3], bytes), per(cy[4], bytes));
        }
        "probe" => {
            // windows of a real dump (raw byte ranges): each whole block in them decoded
            // (its CRC checked), its phases timed
            let mut cy = Cycles::default();
            let mut blocks = 0;
            for path in &args[2..] {
                if path == "slice8" || path == "seg" || path == "simd" { continue; }
                let data = std::fs::read(path).unwrap();
                let magics = block_magics(&data);
                for w in magics.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    let lo = (a / 8) as usize;
                    let hi = ((b / 8) as usize + 16).min(data.len());
                    let mut br = BitReader::new(&data[lo..hi]);
                    br.bits((a % 8) as u32);
                    let mut out = Vec::new();
                    let c = decode_one(&mut br, &mut out, crc_kind);
                    cy.add(&c);
                    blocks += 1;
                }
            }
            let o = cy.out_bytes;
            let total = cy.huffman + cy.mtf + cy.build + cy.walk + cy.unrle_crc;
            println!("{{\"blocks\":{},\"in_bytes\":{},\"out_bytes\":{},\"ratio\":{:.3},\"cy_per_out_byte\":{{\"huffman\":{:.2},\"mtf_rle2\":{:.2},\"build_tt\":{:.2},\"walk\":{:.2},\"unrle_crc\":{:.2},\"total\":{:.2}}}}}",
                     blocks, cy.in_bits / 8, o, o as f64 / (cy.in_bits / 8) as f64, per(cy.huffman, o), per(cy.mtf, o), per(cy.build, o), per(cy.walk, o), per(cy.unrle_crc, o), per(total, o));
        }
        "tail" => {
            // the last phase's two parts apart, over every block's RLE1-coded bytes:
            // RLE1 undone scalar and AVX2; the CRC by byte, sliced by 8, by PCLMULQDQ
            let data = std::fs::read(&args[2]).unwrap();
            let nb: usize = args[3].parse().unwrap();
            let mut prepared = Vec::new();
            let mut out = Vec::new();
            decode_all(&data, &mut out, false, Some(&mut prepared), crc::Kind::Slice8);
            prepared.truncate(nb);
            let rles: Vec<Vec<u8>> = prepared.iter().map(|p| {
                let mut tt = Vec::new(); build(&p.ll, &p.counts, &mut tt);
                let mut r = Vec::new(); walk(&tt, p.orig_ptr, &mut r); r }).collect();
            let rle_bytes: u64 = rles.iter().map(|r| r.len() as u64).sum();
            let mut outs: Vec<Vec<u8>> = vec![Vec::new(); rles.len()];
            let mut o2 = Vec::new();
            let mut cy = [u64::MAX; 5];
            let mut ok = true;
            for _ in 0..3 {
                let t0 = rdtsc();
                for (r, o) in rles.iter().zip(outs.iter_mut()) { o.clear(); simd::unrle_scalar(r, o, 0); }
                cy[0] = cy[0].min(rdtsc() - t0);
                let t0 = rdtsc();
                for (r, o) in rles.iter().zip(outs.iter()) { o2.clear(); simd::unrle(r, &mut o2); ok &= &o2 == o; }
                cy[1] = cy[1].min(rdtsc() - t0);
                let mut c = [0u32; 3];
                let t0 = rdtsc();
                for o in &outs { c[0] ^= crc::crc32_bzip2(o, crc::Kind::Bytewise); }
                cy[2] = cy[2].min(rdtsc() - t0);
                let t0 = rdtsc();
                for o in &outs { c[1] ^= crc::crc32_bzip2(o, crc::Kind::Slice8); }
                cy[3] = cy[3].min(rdtsc() - t0);
                let t0 = rdtsc();
                for o in &outs { c[2] ^= simd::crc32_bzip2(o); }
                cy[4] = cy[4].min(rdtsc() - t0);
                ok &= c[0] == c[1] && c[1] == c[2];
                for (o, p) in outs.iter().zip(&prepared) { ok &= crc::crc32_bzip2(o, crc::Kind::Slice8) == p.crc && simd::crc32_bzip2(o) == p.crc; }
            }
            let ob: u64 = outs.iter().map(|o| o.len() as u64).sum();
            println!("{{\"blocks\":{},\"equal_and_crcs_match\":{},\"cy_per_out_byte\":{{\"unrle_scalar\":{:.3},\"unrle_avx2\":{:.3},\"crc_bytewise\":{:.3},\"crc_slice8\":{:.3},\"crc_pclmul\":{:.3}}},\"rle_bytes\":{},\"out_bytes\":{}}}",
                     rles.len(), ok, per(cy[0], ob), per(cy[1], ob), per(cy[2], ob), per(cy[3], ob), per(cy[4], ob), rle_bytes, ob);
        }
        "probecat" => {
            // the whole blocks of each window, decoded, to stdout (a window's text)
            let stdout = std::io::stdout();
            let mut w = stdout.lock();
            for path in &args[2..] {
                let data = std::fs::read(path).unwrap();
                let magics = block_magics(&data);
                for m in magics.windows(2) {
                    let lo = (m[0] / 8) as usize;
                    let hi = ((m[1] / 8) as usize + 16).min(data.len());
                    let mut br = BitReader::new(&data[lo..hi]);
                    br.bits((m[0] % 8) as u32);
                    let mut out = Vec::new();
                    decode_one(&mut br, &mut out, crc_kind);
                    w.write_all(&out).unwrap();
                }
                w.write_all(b"\n").unwrap();
            }
        }
        "chase" => chase(args.iter().any(|a| a == "huge")),
        "bandwidth" => bandwidth(),
        _ => panic!("unknown mode"),
    }
}

/// Decode to `w` phase by phase (the phases' passes, as `decode_all`); returns the bytes
fn cat_phased(data: &[u8], w: &mut impl Write, crc_kind: crc::Kind) -> u64 {
    let mut br = BitReader::new(data);
    let (mut syms, mut ll, mut tt, mut rle, mut out) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut counts = [0u32; 256];
    let mut n = 0;
    while stream_header(&mut br) {
        let mut combined = 0u32;
        loop {
            match read_block(&mut br, syms) {
                Next::End(crc) => {
                    assert_eq!(crc, combined);
                    syms = Vec::new();
                    break;
                }
                Next::Block(b) => {
                    if crc_kind == crc::Kind::Simd { simd::mtf(&b, &mut ll, &mut counts); } else { mtf(&b, &mut ll, &mut counts); }
                    build(&ll, &counts, &mut tt);
                    do_walk(&tt, b.orig_ptr, &mut rle);
                    out.clear();
                    let crc = unrle_crc(&rle, &mut out, crc_kind);
                    assert_eq!(crc, b.crc);
                    combined = combined.rotate_left(1) ^ crc;
                    w.write_all(&out).unwrap();
                    n += out.len() as u64;
                    syms = b.syms;
                }
            }
        }
        br.align_byte();
    }
    n
}

/// Decode to `w` with the walk, RLE1 and CRC fused as lbzip2's emit
fn cat_fused(data: &[u8], w: &mut impl Write) -> u64 {
    let mut br = BitReader::new(data);
    let (mut syms, mut ll, mut tt, mut out) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut counts = [0u32; 256];
    let mut n = 0;
    while stream_header(&mut br) {
        let mut combined = 0u32;
        loop {
            match read_block(&mut br, syms) {
                Next::End(crc) => {
                    assert_eq!(crc, combined);
                    syms = Vec::new();
                    break;
                }
                Next::Block(b) => {
                    mtf(&b, &mut ll, &mut counts);
                    build(&ll, &counts, &mut tt);
                    out.clear();
                    let crc = walk_unrle_crc_fused(&tt, b.orig_ptr, &mut out);
                    assert_eq!(crc, b.crc);
                    combined = combined.rotate_left(1) ^ crc;
                    w.write_all(&out).unwrap();
                    n += out.len() as u64;
                    syms = b.syms;
                }
            }
        }
        br.align_byte();
    }
    n
}

/// A random cyclic permutation of `n` u32 slots (Sattolo), so a chase visits every slot
fn cycle(n: usize, seed: &mut u64) -> Vec<u32> {
    let mut idx: Vec<u32> = (0..n as u32).collect();
    for i in (1..n).rev() {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        let j = (*seed % i as u64) as usize;
        idx.swap(i, j);
    }
    let mut next = vec![0u32; n];
    for i in 0..n {
        next[idx[i] as usize] = idx[(i + 1) % n];
    }
    next
}

fn chase(huge: bool) {
    let mut seed = 0x9e3779b97f4a7c15u64;
    println!("# working set (KiB), cycles per dependent load (1 chain), and with 2/4/8 chains interleaved (cycles per load)");
    for kib in [16usize, 32, 64, 256, 1024, 1536, 2048, 3072, 3600, 4096, 8192, 16384, 65536, 262144, 1048576] {
        let n = kib * 1024 / 4;
        let next0 = cycle(n, &mut seed);
        let next: &[u32] = if huge { huge_copy(&next0) } else { &next0 };
        let steps = (n * 4).clamp(4 << 20, 32 << 20);
        let mut res = Vec::new();
        for k in [1usize, 2, 4, 8] {
            let mut p: Vec<u32> = (0..k).map(|j| (j * n / k) as u32).collect();
            let per_chain = steps / k;
            let t0 = rdtsc();
            for _ in 0..per_chain {
                for q in p.iter_mut() {
                    *q = next[*q as usize];
                }
            }
            let c = rdtsc() - t0;
            std::hint::black_box(&p);
            res.push(format!("{:.2}", c as f64 / (per_chain * k) as f64));
        }
        println!("{{\"kib\":{},\"huge\":{},\"anon_huge_kb\":{},\"cy_per_load\":[{}]}}", kib, huge, anon_huge_kb(), res.join(","));
    }
}

fn bandwidth() {
    let n = 1usize << 28; // 1 GiB of u32 per buffer... as u64: 2 GiB; keep 512 MiB
    let words = n / 4; // 64M u64 = 512 MiB
    for threads in 1..=4usize {
        let per = words / threads;
        let src: Vec<u64> = (0..words as u64).collect();
        let mut dst = vec![0u64; words];
        // read
        let mut best_r = f64::MAX;
        let mut best_c = f64::MAX;
        for _ in 0..3 {
            let t = Instant::now();
            std::thread::scope(|s| {
                for c in src.chunks(per) {
                    s.spawn(move || {
                        let mut acc = 0u64;
                        for x in c {
                            acc = acc.wrapping_add(*x);
                        }
                        std::hint::black_box(acc);
                    });
                }
            });
            best_r = best_r.min(t.elapsed().as_secs_f64());
            let t = Instant::now();
            std::thread::scope(|s| {
                for (d, c) in dst.chunks_mut(per).zip(src.chunks(per)) {
                    s.spawn(move || d.copy_from_slice(c));
                }
            });
            best_c = best_c.min(t.elapsed().as_secs_f64());
        }
        let bytes = (words * 8) as f64;
        println!("{{\"threads\":{},\"read_GB_s\":{:.2},\"copy_GB_s_read_plus_write\":{:.2}}}", threads,
                 bytes / best_r / 1e9, 2.0 * bytes / best_c / 1e9);
    }
}

fn anon_huge_kb() -> u64 {
    std::fs::read_to_string("/proc/self/smaps_rollup").ok().and_then(|s| {
        s.lines().find(|l| l.starts_with("AnonHugePages:")).and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
    }).unwrap_or(0)
}

/// Every bit offset of a block's magic (0x314159265359) in `data`
fn block_magics(data: &[u8]) -> Vec<u64> {
    let mut out = Vec::new();
    for i in 0..data.len().saturating_sub(8) {
        let w = u64::from_be_bytes(data[i..i + 8].try_into().unwrap());
        for s in 0..8 {
            if (w >> (16 - s)) & 0xffff_ffff_ffff == 0x3141_5926_5359 {
                out.push(i as u64 * 8 + s as u64);
            }
        }
    }
    out
}
