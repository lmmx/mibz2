#![forbid(unsafe_code)]
//! Smooth-zone `explore` probe (zero `src/` edits, `#![forbid(unsafe_code)]`):
//! Track 1: Inline `Seq(OpenCells)` & `Seq(OpenR2Cells)` vs shipped `Seq(Cells)` plus
//!          `t = 2, 3` worker pools (`Cells` vs `OpenCells` vs `OpenR2Cells`), warm & L3-cold,
//!          measuring steal-free CPU, wall, true subprocess peak RSS (`VmHWM`), and SHA-256 parity.
//! Track 2: Micro-dissection of `src/cells.rs` (`EagerCells`) vs `OpenCells` vs `OpenR2Cells`
//!          (route/bucket breakdown, buffer-footprint ledger, phase timings including `owned-writer`).
//!
//! # Usage
//! ```bash
//! cargo build --release --features "moonshot trace owned-writer" --example eager_cells_seq_open_probe
//! # Optional future Open features (#1110 forward-anchor-lanes):
//! cargo build --release --features "moonshot forward-anchor-lanes" --example eager_cells_seq_open_probe
//!
//! eager_cells_seq_open_probe child-seq <cells|open|open-r2|seg> <FILE.bz2> [--cold] [--sha256]
//! eager_cells_seq_open_probe child-par <cells|open|open-r2|seg> <THREADS> <FILE.bz2> [--sha256]
//! eager_cells_seq_open_probe dissect   <FILE.bz2> [--reps 5] [--cold]
//! ```

#[cfg(feature = "moonshot")]
mod imp {
    use mibz2::block::Symbols;
    use mibz2::bwt::{Cells, InverseBwt, OpenCells, Segmented};
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::{crc, decode_block, mtf, rle, BzReader, Kernels, Scratch};
    use std::hint::black_box;
    use std::io::{Cursor, Read};
    use std::time::Instant;

    /// Minimal self-contained SHA-256 for byte-exact parity against `libbzip2`.
    struct Sha256 {
        h: [u32; 8],
        buf: [u8; 64],
        len: usize,
        total: u64,
    }

    impl Sha256 {
        fn new() -> Self {
            Self {
                h: [
                    0x6a09_e667,
                    0xbb67_ae85,
                    0x3c6e_f372,
                    0xa54f_f53a,
                    0x510e_527f,
                    0x9b05_688c,
                    0x1f83_d9ab,
                    0x5be0_cd19,
                ],
                buf: [0; 64],
                len: 0,
                total: 0,
            }
        }

        fn compress(&mut self, block: &[u8; 64]) {
            const K: [u32; 64] = [
                0x428a_2f98,
                0x7137_4491,
                0xb5c0_fbcf,
                0xe9b5_dba5,
                0x3956_c25b,
                0x59f1_11f1,
                0x923f_82a4,
                0xab1c_5ed5,
                0xd807_aa98,
                0x1283_5b01,
                0x2431_85be,
                0x550c_7dc3,
                0x72be_5d74,
                0x80de_b1fe,
                0x9bdc_06a7,
                0xc19b_f174,
                0xe49b_69c1,
                0xefbe_4786,
                0x0fc1_9dc6,
                0x240c_a1cc,
                0x2de9_2c6f,
                0x4a74_84aa,
                0x5cb0_a9dc,
                0x76f9_88da,
                0x983e_5152,
                0xa831_c66d,
                0xb003_27c8,
                0xbf59_7fc7,
                0xc6e0_0bf3,
                0xd5a7_9147,
                0x06ca_6351,
                0x1429_2967,
                0x27b7_0a85,
                0x2e1b_2138,
                0x4d2c_6dfc,
                0x5338_0d13,
                0x650a_7354,
                0x766a_0abb,
                0x81c2_c92e,
                0x9272_2c85,
                0xa2bf_e8a1,
                0xa81a_664b,
                0xc24b_8b70,
                0xc76c_51a3,
                0xd192_e819,
                0xd699_0624,
                0xf40e_3585,
                0x106a_a070,
                0x19a4_c116,
                0x1e37_6c08,
                0x2748_774c,
                0x34b0_bcb5,
                0x391c_0cb3,
                0x4ed8_aa4a,
                0x5b9c_ca4f,
                0x682e_6ff3,
                0x748f_82ee,
                0x78a5_636f,
                0x84c8_7814,
                0x8cc7_0208,
                0x90be_fffa,
                0xa450_6ceb,
                0xbef9_a3f7,
                0xc671_78f2,
            ];
            let mut w = [0u32; 64];
            for i in 0..16 {
                w[i] = u32::from_be_bytes([
                    block[4 * i],
                    block[4 * i + 1],
                    block[4 * i + 2],
                    block[4 * i + 3],
                ]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }
            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.h;
            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let t1 = h
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[i])
                    .wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let t2 = s0.wrapping_add(maj);
                h = g;
                g = f;
                f = e;
                e = d.wrapping_add(t1);
                d = c;
                c = b;
                b = a;
                a = t1.wrapping_add(t2);
            }
            for (dst, add) in self.h.iter_mut().zip([a, b, c, d, e, f, g, h]) {
                *dst = dst.wrapping_add(add);
            }
        }

        fn update(&mut self, mut data: &[u8]) {
            self.total += data.len() as u64;
            if self.len > 0 {
                let take = (64 - self.len).min(data.len());
                self.buf[self.len..self.len + take].copy_from_slice(&data[..take]);
                self.len += take;
                data = &data[take..];
                if self.len == 64 {
                    let b = self.buf;
                    self.compress(&b);
                    self.len = 0;
                }
            }
            while data.len() >= 64 {
                let b: [u8; 64] = data[..64].try_into().unwrap();
                self.compress(&b);
                data = &data[64..];
            }
            if !data.is_empty() {
                self.buf[..data.len()].copy_from_slice(data);
                self.len = data.len();
            }
        }

        fn finish_hex(mut self) -> String {
            let bits = self.total * 8;
            self.update(&[0x80]);
            if self.len > 56 {
                let zeros = [0u8; 64];
                self.update(&zeros[..64 - self.len]);
            }
            let zeros = [0u8; 56];
            self.update(&zeros[..56 - self.len]);
            self.update(&bits.to_be_bytes());
            let mut s = String::with_capacity(64);
            for w in self.h {
                s.push_str(&format!("{w:08x}"));
            }
            s
        }
    }

    /// Exact post-#1080 `R2_2tier_m22` pre-route check (from `route-crossover-post-1080.md`):
    /// - `1/64 <= r/n < 0.10`: outright Open (0 ns scan).
    /// - `0.10 <= r/n < 0.15`: `within_cap(m <= min(256_000, 0.22*n), strips <= 16_384)`.
    /// - `r/n >= 0.15` or `< 1/64`: straight to `Segmented` (no dense band, no wasted `within_cap`).
    fn r2_admits_open(records: &[u16], n: usize, p: u32) -> bool {
        if n == 0 || n > 900_000 || p as usize >= n {
            return false;
        }
        let r = records.len();
        if r * 64 < n {
            return false;
        }
        if r * 10 < n {
            return true;
        }
        if r * 20 >= n * 3 {
            return false;
        }
        // 0.10 <= r/n < 0.15: live `within_cap` with m <= min(256_000, 0.22*n), strips <= 16_384.
        let cap_m = 256_000u32.min((n as u64 * 22 / 100) as u32);
        let cap_strips = 16_384usize;
        let (mut pos, mut m, mut ns) = (0u32, 0u32, 0usize);
        let mut i = 0;
        while i < records.len() {
            let rec = records[i];
            let c = rec >> 8;
            let mut width = u32::from(rec & 0xFF);
            i += 1;
            while i < records.len() && (records[i] >> 8) == c {
                width += u32::from(records[i] & 0xFF);
                i += 1;
            }
            if width <= 16 {
                m += width;
            } else if p.wrapping_sub(pos) >= width {
                ns += 1;
            } else {
                m += 1;
                ns += usize::from(pos < p) + usize::from(p + 1 < pos + width);
            }
            if m > cap_m || ns > cap_strips {
                return false;
            }
            pos += width;
        }
        true
    }

    /// `OpenCells` gated by post-#1080 `R2_2tier_m22`, routing refused blocks straight to `Segmented`
    /// (zero `src/` edits; exercises the exact shipped `OpenCells` and `Segmented` kernels).
    #[derive(Default)]
    struct OpenR2Cells {
        open: OpenCells,
        seg: Segmented,
    }

    impl InverseBwt for OpenR2Cells {
        fn reads_runs(&self) -> bool {
            true
        }
        fn invert_runs(
            &mut self,
            r: &Runs,
            n: usize,
            p: u32,
            out: &mut Vec<u8>,
            k: Kernels,
        ) -> bool {
            if r2_admits_open(r.records(), n, p) {
                self.open.invert_runs(r, n, p, out, k)
            } else {
                self.seg.invert_runs(r, n, p, out, k)
            }
        }
        fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
            self.seg.invert(ll, counts, p, out);
        }
        fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
            self.seg.invert_in_place(ll, counts, p)
        }
        #[cfg(feature = "trace")]
        fn buffers(&self) -> Vec<mibz2::trace::Buffer> {
            let mut v = self.open.buffers();
            v.extend(self.seg.buffers());
            v
        }
    }

    fn thread_cpu_ns() -> u64 {
        if let Ok(s) = std::fs::read_to_string("/proc/thread-self/schedstat") {
            if let Some(first) = s.split_whitespace().next() {
                if let Ok(v) = first.parse::<u64>() {
                    return v;
                }
            }
        }
        0
    }

    /// Process-wide CPU (utime + stime in seconds) from `/proc/self/stat` (100 Hz tick).
    fn proc_cpu_s() -> f64 {
        if let Ok(s) = std::fs::read_to_string("/proc/self/stat") {
            if let Some(rp) = s.rfind(')') {
                let f: Vec<&str> = s[rp + 2..].split_whitespace().collect();
                if f.len() > 12 {
                    let ut: u64 = f[11].parse().unwrap_or(0);
                    let st: u64 = f[12].parse().unwrap_or(0);
                    return (ut + st) as f64 / 100.0;
                }
            }
        }
        0.0
    }

    /// True process peak RSS (`VmHWM`) in KiB from `/proc/self/status`.
    fn vm_hwm_kb() -> u64 {
        if let Ok(s) = std::fs::read_to_string("/proc/self/status") {
            for line in s.lines() {
                if let Some(rest) = line.strip_prefix("VmHWM:") {
                    if let Some(tok) = rest.split_whitespace().next() {
                        if let Ok(v) = tok.parse::<u64>() {
                            return v;
                        }
                    }
                }
            }
        }
        0
    }

    /// Flush 64 MiB of L3 cache lines (returns checksum so LLVM cannot elide).
    fn flush_l3(buf: &mut [u8]) -> u64 {
        let mut acc = 0u64;
        for i in (0..buf.len()).step_by(64) {
            let v = buf[i].wrapping_add(1);
            buf[i] = v;
            acc = acc.wrapping_add(v as u64);
        }
        black_box(acc)
    }

    /// Streaming inline single-threaded decode (`Scanner` + `decode_block::<I>`), matching
    /// `BzReader::new`'s exact streaming shape (zero thread spawn, zero whole-file output Vec).
    fn run_seq<I: InverseBwt>(
        data: &[u8],
        k: Kernels,
        cold_buf: &mut Option<Vec<u8>>,
        do_sha: bool,
    ) -> (f64, f64, usize, u32, Option<String>) {
        let mut scratch = Scratch::<I>::default();
        let mut sc = Scanner::new(SliceSource(data));
        let mut syms = Vec::new();
        let mut out = Vec::new();
        let mut total_bytes = 0usize;
        let mut total_crc = 0u32;
        let mut sha = if do_sha { Some(Sha256::new()) } else { None };

        if let Some(cb) = cold_buf.as_mut() {
            flush_l3(cb);
        }
        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let mut flush_wall = 0.0f64;
        let mut flush_cpu = 0u64;
        loop {
            let ev = sc.next(std::mem::take(&mut syms)).expect("scan");
            match ev {
                Event::Block { symbols: b, .. } => {
                    if let Some(cb) = cold_buf.as_mut() {
                        let fc0 = thread_cpu_ns();
                        let fw0 = Instant::now();
                        flush_l3(cb);
                        flush_wall += fw0.elapsed().as_secs_f64();
                        flush_cpu += thread_cpu_ns().saturating_sub(fc0);
                    }
                    out.clear();
                    decode_block(&b, k, &mut scratch, &mut out).expect("decode_block");
                    total_bytes += out.len();
                    total_crc = total_crc.rotate_left(1) ^ b.crc;
                    if let Some(ref mut s) = sha {
                        s.update(&out);
                    }
                    syms = b.syms;
                }
                Event::Done => break,
            }
        }
        let wall = (w0.elapsed().as_secs_f64() - flush_wall).max(0.0);
        let cpu = (thread_cpu_ns().saturating_sub(c0).saturating_sub(flush_cpu)) as f64 / 1e9;
        let hex = sha.map(Sha256::finish_hex);
        (wall, cpu, total_bytes, total_crc, hex)
    }

    fn cmd_child_seq(arm: &str, path: &str, cold: bool, do_sha: bool) {
        let data = std::fs::read(path).expect("read bz2");
        let k = Kernels::detect();
        let mut cold_buf = if cold {
            Some(vec![0x5Au8; 64 * 1024 * 1024])
        } else {
            None
        };
        let (wall, cpu, bytes, crc, sha) = match arm {
            "cells" => run_seq::<Cells>(&data, k, &mut cold_buf, do_sha),
            "open" => run_seq::<OpenCells>(&data, k, &mut cold_buf, do_sha),
            "open-r2" => run_seq::<OpenR2Cells>(&data, k, &mut cold_buf, do_sha),
            "seg" => run_seq::<Segmented>(&data, k, &mut cold_buf, do_sha),
            _ => panic!("unknown arm: {arm}"),
        };
        let rss_kb = vm_hwm_kb();
        let sha_json = sha.map_or_else(|| "null".to_string(), |h| format!("\"{h}\""));
        println!(
            "{{\"mode\":\"seq\",\"arm\":\"{arm}\",\"cold\":{cold},\"wall\":{wall:.6},\"cpu\":{cpu:.6},\"rss_kb\":{rss_kb},\"out\":{bytes},\"crc\":\"{crc:08x}\",\"sha256\":{sha_json}}}"
        );
    }

    fn run_par<I: InverseBwt + 'static>(
        data: Vec<u8>,
        threads: usize,
        do_sha: bool,
    ) -> (f64, f64, usize, Option<String>) {
        let c0 = proc_cpu_s();
        let w0 = Instant::now();
        let mut r = BzReader::parallel_workers_with::<I>(
            Cursor::new(data),
            threads,
            4 * threads,
            Kernels::detect(),
        );
        let mut buf = vec![0u8; 64 * 1024];
        let mut total = 0usize;
        let mut sha = if do_sha { Some(Sha256::new()) } else { None };
        loop {
            let n = r.read(&mut buf).expect("read");
            if n == 0 {
                break;
            }
            total += n;
            if let Some(ref mut s) = sha {
                s.update(&buf[..n]);
            }
        }
        drop(r);
        let wall = w0.elapsed().as_secs_f64();
        let cpu = (proc_cpu_s() - c0).max(0.0);
        let hex = sha.map(Sha256::finish_hex);
        (wall, cpu, total, hex)
    }

    fn cmd_child_par(arm: &str, threads: usize, path: &str, do_sha: bool) {
        let data = std::fs::read(path).expect("read bz2");
        let (wall, cpu, bytes, sha) = match arm {
            "cells" => run_par::<Cells>(data, threads, do_sha),
            "open" => run_par::<OpenCells>(data, threads, do_sha),
            "open-r2" => run_par::<OpenR2Cells>(data, threads, do_sha),
            "seg" => run_par::<Segmented>(data, threads, do_sha),
            _ => panic!("unknown arm: {arm}"),
        };
        let rss_kb = vm_hwm_kb();
        let sha_json = sha.map_or_else(|| "null".to_string(), |h| format!("\"{h}\""));
        println!(
            "{{\"mode\":\"par\",\"arm\":\"{arm}\",\"threads\":{threads},\"wall\":{wall:.6},\"cpu\":{cpu:.6},\"rss_kb\":{rss_kb},\"out\":{bytes},\"sha256\":{sha_json}}}"
        );
    }

    fn bucket_of(b: &Symbols, r: &Runs, n: usize) -> &'static str {
        if !runs::takes_runs(b) {
            return "Column";
        }
        let recs = r.records().len();
        if recs * 64 < n {
            "Sparse(<1/64)"
        } else if recs * 10 < n {
            "Low(<0.10)"
        } else if recs * 20 < n * 3 {
            if r2_admits_open(r.records(), n, b.orig_ptr) {
                "MidAdmit(0.10..0.15)"
            } else {
                "MidRef(0.10..0.15)"
            }
        } else if recs * 25 < n * 7 {
            "Dense(0.15..0.28)"
        } else {
            "High(>=0.28)"
        }
    }

    fn cmd_dissect(path: &str, reps: usize, cold: bool) {
        let data = std::fs::read(path).expect("read bz2");
        let k = Kernels::detect();
        let mut sc = Scanner::new(SliceSource(&data));
        let mut blocks: Vec<Symbols> = Vec::new();
        while let Event::Block { symbols: b, .. } = sc.next(Vec::new()).expect("scan") {
            blocks.push(b);
        }

        // Pre-compute route buckets & verify exactness across all 4 kernels.
        let mut runs_buf = Runs::default();
        let mut counts = [0u32; 256];
        let mut ll = Vec::new();
        let mut out_c = Vec::new();
        let mut out_o = Vec::new();
        let mut out_r2 = Vec::new();
        let mut out_s = Vec::new();

        let mut sc_c = Scratch::<Cells>::default();
        let mut sc_o = Scratch::<OpenCells>::default();
        let mut sc_r2 = Scratch::<OpenR2Cells>::default();
        let mut sc_s = Scratch::<Segmented>::default();

        let mut bkt_names: Vec<&'static str> = Vec::with_capacity(blocks.len());
        let mut bkt_counts: std::collections::BTreeMap<&'static str, (usize, usize)> =
            std::collections::BTreeMap::new();

        for b in &blocks {
            let n = if runs::takes_runs(b) {
                runs::front(b, &mut runs_buf, k).expect("front")
            } else {
                mtf::mtf(b, &mut ll, &mut counts, k).expect("mtf")
            };
            let bkt = bucket_of(b, &runs_buf, n);
            bkt_names.push(bkt);
            let e = bkt_counts.entry(bkt).or_insert((0, 0));
            e.0 += 1;
            e.1 += n;

            out_c.clear();
            out_o.clear();
            out_r2.clear();
            out_s.clear();
            decode_block(b, k, &mut sc_c, &mut out_c).expect("cells");
            decode_block(b, k, &mut sc_o, &mut out_o).expect("open");
            decode_block(b, k, &mut sc_r2, &mut out_r2).expect("open-r2");
            decode_block(b, k, &mut sc_s, &mut out_s).expect("seg");
            assert_eq!(out_c, out_s, "cells vs seg");
            assert_eq!(out_o, out_s, "open vs seg");
            assert_eq!(out_r2, out_s, "open-r2 vs seg");
        }

        println!(
            "=== DISSECT {path} (blocks={}, cold={cold}, reps={reps}) ===",
            blocks.len()
        );
        println!("Cells counts: {:?}", sc_c.cells_counts());
        println!("Bucket breakdown:");
        for (k_bkt, (nb, rows)) in &bkt_counts {
            println!("  {k_bkt:22}: {nb:4} blks, {rows:10} rows");
        }

        #[cfg(feature = "trace")]
        {
            let dump_bufs = |label: &str, bufs: Vec<mibz2::trace::Buffer>| {
                let total_cap: usize = bufs.iter().map(|b| b.cap).sum();
                let total_len: usize = bufs.iter().map(|b| b.len).sum();
                let parts: Vec<String> = bufs
                    .iter()
                    .filter(|b| b.cap > 0)
                    .map(|b| format!("{}={:.1}K", b.name, b.cap as f64 / 1024.0))
                    .collect();
                println!(
                    "  Buffer ledger [{label:8}]: cap={:.2} MiB, len={:.2} MiB ({})",
                    total_cap as f64 / (1024.0 * 1024.0),
                    total_len as f64 / (1024.0 * 1024.0),
                    parts.join(", ")
                );
            };
            println!("Scratch buffer footprints (after full file):");
            dump_bufs("Cells", sc_c.buffers());
            dump_bufs("Open", sc_o.buffers());
            dump_bufs("OpenR2", sc_r2.buffers());
            dump_bufs("Seg", sc_s.buffers());
        }

        #[cfg(feature = "owned-writer")]
        {
            let ph = mibz2::owned_phases();
            println!(
                "Cells owned-writer phases (1 pass): full_period={}, fallback={}, rules={}, commands={}, scan={:.2}ms, copy={:.2}ms",
                ph[0],
                ph[1],
                ph[2],
                ph[3],
                ph[4] as f64 / 1e6,
                ph[5] as f64 / 1e6
            );
        }

        // Now time per-bucket `invert_runs` / whole `decode_block` across `reps` interleaved rounds.
        let mut cold_buf = if cold {
            Some(vec![0x5Au8; 64 * 1024 * 1024])
        } else {
            None
        };

        let mut tot_cells = Vec::new();
        let mut tot_open = Vec::new();
        let mut tot_r2 = Vec::new();
        let mut tot_seg = Vec::new();
        let mut front_ms = Vec::new();
        let mut rle_crc_ms = Vec::new();

        // Also track Cells vs Open vs OpenR2 vs Seg summed by bucket (last rep or median of reps).
        let mut bkt_c: std::collections::BTreeMap<&'static str, Vec<f64>> = Default::default();
        let mut bkt_o: std::collections::BTreeMap<&'static str, Vec<f64>> = Default::default();
        let mut bkt_r2: std::collections::BTreeMap<&'static str, Vec<f64>> = Default::default();
        let mut bkt_s: std::collections::BTreeMap<&'static str, Vec<f64>> = Default::default();

        // Pre-build MTF/front + raw post-walk bytes for isolated RLE1+CRC & front timing.
        for r in 0..reps {
            let order: [usize; 4] = match r % 4 {
                0 => [0, 1, 2, 3],
                1 => [1, 2, 3, 0],
                2 => [2, 3, 0, 1],
                _ => [3, 0, 1, 2],
            };
            let mut rep_bkt: [std::collections::BTreeMap<&'static str, u64>; 4] =
                Default::default();
            let mut rep_tot = [0u64; 4];

            for &arm in &order {
                if let Some(cb) = cold_buf.as_mut() {
                    flush_l3(cb);
                }
                for (bi, b) in blocks.iter().enumerate() {
                    if let Some(cb) = cold_buf.as_mut() {
                        flush_l3(cb);
                    }
                    let c0 = thread_cpu_ns();
                    match arm {
                        0 => {
                            out_c.clear();
                            decode_block(b, k, &mut sc_c, &mut out_c).unwrap();
                            black_box(&out_c);
                        }
                        1 => {
                            out_o.clear();
                            decode_block(b, k, &mut sc_o, &mut out_o).unwrap();
                            black_box(&out_o);
                        }
                        2 => {
                            out_r2.clear();
                            decode_block(b, k, &mut sc_r2, &mut out_r2).unwrap();
                            black_box(&out_r2);
                        }
                        _ => {
                            out_s.clear();
                            decode_block(b, k, &mut sc_s, &mut out_s).unwrap();
                            black_box(&out_s);
                        }
                    }
                    let dt = thread_cpu_ns().saturating_sub(c0);
                    rep_tot[arm] += dt;
                    *rep_bkt[arm].entry(bkt_names[bi]).or_insert(0) += dt;
                }
            }
            tot_cells.push(rep_tot[0] as f64 / 1e6);
            tot_open.push(rep_tot[1] as f64 / 1e6);
            tot_r2.push(rep_tot[2] as f64 / 1e6);
            tot_seg.push(rep_tot[3] as f64 / 1e6);
            for &bkt in bkt_counts.keys() {
                bkt_c
                    .entry(bkt)
                    .or_default()
                    .push(*rep_bkt[0].get(bkt).unwrap_or(&0) as f64 / 1e6);
                bkt_o
                    .entry(bkt)
                    .or_default()
                    .push(*rep_bkt[1].get(bkt).unwrap_or(&0) as f64 / 1e6);
                bkt_r2
                    .entry(bkt)
                    .or_default()
                    .push(*rep_bkt[2].get(bkt).unwrap_or(&0) as f64 / 1e6);
                bkt_s
                    .entry(bkt)
                    .or_default()
                    .push(*rep_bkt[3].get(bkt).unwrap_or(&0) as f64 / 1e6);
            }

            // Also isolate MTF/front and RLE1+CRC on this rep.
            let c0 = thread_cpu_ns();
            for b in &blocks {
                if runs::takes_runs(b) {
                    black_box(runs::front(b, &mut runs_buf, k).unwrap());
                } else {
                    black_box(mtf::mtf(b, &mut ll, &mut counts, k).unwrap());
                }
            }
            front_ms.push(thread_cpu_ns().saturating_sub(c0) as f64 / 1e6);

            let mut rle_ns = 0u64;
            let mut rle_out = Vec::new();
            let mut raw = Vec::new();
            let mut seg_tmp = Segmented::default();
            for b in &blocks {
                raw.clear();
                if runs::takes_runs(b) {
                    let n = runs::front(b, &mut runs_buf, k).unwrap();
                    seg_tmp.invert_runs(&runs_buf, n, b.orig_ptr, &mut raw, k);
                } else {
                    mtf::mtf(b, &mut ll, &mut counts, k).unwrap();
                    seg_tmp.invert(&ll, &counts, b.orig_ptr, &mut raw);
                }
                let rc0 = thread_cpu_ns();
                rle_out.clear();
                rle::unrle(&raw, &mut rle_out, k).unwrap();
                let c = crc::crc32(&rle_out, k);
                assert_eq!(c, b.crc);
                black_box(&rle_out);
                rle_ns += thread_cpu_ns().saturating_sub(rc0);
            }
            rle_crc_ms.push(rle_ns as f64 / 1e6);
        }

        let med = |v: &Vec<f64>| {
            let mut s = v.clone();
            s.sort_by(|a, b| a.total_cmp(b));
            s[s.len() / 2]
        };
        let mm = |v: &Vec<f64>| {
            let lo = v.iter().copied().fold(f64::INFINITY, f64::min);
            let hi = v.iter().copied().fold(0.0f64, f64::max);
            (lo, hi)
        };
        let (mc, mo, mr2, ms) = (med(&tot_cells), med(&tot_open), med(&tot_r2), med(&tot_seg));
        let (mf, mrc) = (med(&front_ms), med(&rle_crc_ms));
        println!("\nWhole decode_block CPU medians (ms, steal-free schedstat):");
        println!("  MTF/front : {mf:7.2} ms  |  RLE1+CRC: {mrc:7.2} ms");
        println!(
            "  Cells     : {mc:7.2} ms [{:.2}..{:.2}]  (est invBWT={:6.2} ms)",
            mm(&tot_cells).0,
            mm(&tot_cells).1,
            (mc - mf - mrc).max(0.0)
        );
        println!(
            "  Open      : {mo:7.2} ms [{:.2}..{:.2}]  ({:+.2}% vs Cells, est invBWT={:6.2} ms)",
            mm(&tot_open).0,
            mm(&tot_open).1,
            (mo / mc - 1.0) * 100.0,
            (mo - mf - mrc).max(0.0)
        );
        println!(
            "  OpenR2    : {mr2:7.2} ms [{:.2}..{:.2}]  ({:+.2}% vs Cells, est invBWT={:6.2} ms)",
            mm(&tot_r2).0,
            mm(&tot_r2).1,
            (mr2 / mc - 1.0) * 100.0,
            (mr2 - mf - mrc).max(0.0)
        );
        println!(
            "  Segmented : {ms:7.2} ms [{:.2}..{:.2}]  ({:+.2}% vs Cells, est invBWT={:6.2} ms)",
            mm(&tot_seg).0,
            mm(&tot_seg).1,
            (ms / mc - 1.0) * 100.0,
            (ms - mf - mrc).max(0.0)
        );

        println!("\nPer-bucket decode_block CPU medians (ms):");
        println!(
            "  {:22} | {:>8} | {:>8} ({:>7}) | {:>8} ({:>7}) | {:>8} ({:>7})",
            "bucket", "Cells", "Open", "vs C", "OpenR2", "vs C", "Seg", "vs C"
        );
        for &bkt in bkt_counts.keys() {
            let c = med(&bkt_c[bkt]);
            let o = med(&bkt_o[bkt]);
            let r2 = med(&bkt_r2[bkt]);
            let s = med(&bkt_s[bkt]);
            let pct = |x: f64| if c > 0.01 { (x / c - 1.0) * 100.0 } else { 0.0 };
            println!(
                "  {bkt:22} | {c:8.2} | {o:8.2} ({:+6.2}%) | {r2:8.2} ({:+6.2}%) | {s:8.2} ({:+6.2}%)",
                pct(o),
                pct(r2),
                pct(s)
            );
        }
    }

    pub fn run() {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 3 {
            eprintln!("Usage:");
            eprintln!(
                "  {} child-seq <cells|open|open-r2|seg> <FILE.bz2> [--cold] [--sha256]",
                args[0]
            );
            eprintln!(
                "  {} child-par <cells|open|open-r2|seg> <THREADS> <FILE.bz2> [--sha256]",
                args[0]
            );
            eprintln!("  {} dissect   <FILE.bz2> [--reps 5] [--cold]", args[0]);
            std::process::exit(2);
        }
        match args[1].as_str() {
            "child-seq" => {
                let arm = &args[2];
                let path = &args[3];
                let cold = args.iter().any(|a| a == "--cold");
                let sha = args.iter().any(|a| a == "--sha256");
                cmd_child_seq(arm, path, cold, sha);
            }
            "child-par" => {
                let arm = &args[2];
                let threads: usize = args[3].parse().expect("threads");
                let path = &args[4];
                let sha = args.iter().any(|a| a == "--sha256");
                cmd_child_par(arm, threads, path, sha);
            }
            "dissect" => {
                let path = &args[2];
                let reps = args
                    .windows(2)
                    .find(|w| w[0] == "--reps")
                    .map_or(5, |w| w[1].parse().expect("reps"));
                let cold = args.iter().any(|a| a == "--cold");
                cmd_dissect(path, reps, cold);
            }
            other => panic!("unknown subcommand: {other}"),
        }
    }
}

fn main() {
    #[cfg(feature = "moonshot")]
    imp::run();
    #[cfg(not(feature = "moonshot"))]
    {
        eprintln!(
            "Build with --features moonshot (plus optional trace / owned-writer / forward-anchor-lanes)."
        );
        std::process::exit(2);
    }
}
