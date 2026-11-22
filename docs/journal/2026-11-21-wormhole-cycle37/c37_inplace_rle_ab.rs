use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BzReader, Error, ErrorKind, Kernels, Scratch};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::stream::{Event, Scanner, SliceSource, Source};

// ---- Cycle 37: Emit-Direct-to-Out (0 B scratch.ll) + Exact In-Place RLE1 + L2-Tiled GF(2) CLMUL CRC ----
mod safe_c37 {
    #![forbid(unsafe_code)]
    use super::*;

    const POLY: u32 = 0x04C1_1DB7;

    /// Multiply `v` by `x` mod P (MSB-first CRC-32/BZIP2 polynomial)
    #[inline]
    const fn gf2_mul_x(v: u32) -> u32 {
        if v & 0x8000_0000 != 0 { (v << 1) ^ POLY } else { v << 1 }
    }

    /// Apply 32x32 GF(2) matrix `mat` (where mat[i] = bit_i * x^k mod P) to vector `vec`
    #[inline]
    fn gf2_matrix_times(mat: &[u32; 32], mut vec: u32) -> u32 {
        let mut sum = 0u32;
        let mut i = 0;
        while vec != 0 {
            if vec & 1 != 0 { sum ^= mat[i]; }
            vec >>= 1;
            i += 1;
        }
        sum
    }

    /// Square a 32x32 GF(2) matrix
    const fn gf2_matrix_square(mat: &[u32; 32]) -> [u32; 32] {
        let mut out = [0u32; 32];
        let mut n = 0;
        while n < 32 {
            let mut vec = mat[n];
            let mut sum = 0u32;
            let mut i = 0;
            while vec != 0 {
                if vec & 1 != 0 { sum ^= mat[i]; }
                vec >>= 1;
                i += 1;
            }
            out[n] = sum;
            n += 1;
        }
        out
    }

    /// Build GF(2) matrix for multiplying by x^(8 * bytes) mod P (MSB-first CRC-32/BZIP2)
    /// Precompute tables for bit-powers of byte length `2^k` bytes (k = 0..24, up to 16 MiB)
    const fn build_pow_mats() -> [[u32; 32]; 24] {
        // M_bit: multiplying a 32-bit state by x^1 mod P
        let mut m1 = [0u32; 32];
        let mut i = 0;
        while i < 32 {
            m1[i] = gf2_mul_x(1u32 << i);
            i += 1;
        }
        // M_byte = M_bit^8 (x^8 mod P)
        let m2 = gf2_matrix_square(&m1);
        let m4 = gf2_matrix_square(&m2);
        let mut cur = gf2_matrix_square(&m4);
        let mut mats = [[0u32; 32]; 24];
        let mut k = 0;
        while k < 24 {
            mats[k] = cur;
            cur = gf2_matrix_square(&cur);
            k += 1;
        }
        mats
    }

    static POW_MATS: [[u32; 32]; 24] = build_pow_mats();

    /// Combine raw (un-inverted) CRC-32/BZIP2 state `s1` (from initial 0 or !0) with
    /// `crc2_zero = !crc32_from_zero(chunk)`?
    /// Exact algebraic identity for MSB-first CRC-32/BZIP2:
    /// Let R(s, B) = update(s, B) = s * x^{8|B|} + R(0, B) (mod P).
    /// Note mibz2::crc::crc32(B, k) returns !R(!0, B) = !(!0 * x^{8|B|} + R(0, B)).
    /// Therefore R(0, B) = (!crc32(B, k)) ^ (!0 * x^{8|B|} mod P)!
    /// And thus R(s1, B) = (s1 ^ !0) * x^{8|B|} ^ (!crc32(B, k))!
    /// WAIT — CHECK THAT IDENTITY!
    /// R(s1, B) = s1 * x^{8|B|} ^ R(0, B)
    ///          = s1 * x^{8|B|} ^ (!0 * x^{8|B|}) ^ (!crc32(B, k))
    ///          = (s1 ^ !0) * x^{8|B|} ^ (!crc32(B, k))!!
    /// When s1 = !0 (first chunk), (s1 ^ !0) = 0 -> R(!0, B) = !crc32(B, k)! EXACT!
    #[inline]
    pub fn combine_crc_state(s1: u32, chunk_crc: u32, mut len: usize) -> u32 {
        let mut v = s1 ^ !0u32;
        if v != 0 {
            let mut bit = 0usize;
            while len != 0 {
                if len & 1 != 0 {
                    v = gf2_matrix_times(&POW_MATS[bit], v);
                }
                len >>= 1;
                bit += 1;
            }
        }
        v ^ !chunk_crc
    }

    /// Fast 8-byte SWAR 4-equal-byte finder
    #[inline]
    pub fn find_run4_swar(s: &[u8], mut i: usize) -> Option<usize> {
        let n = s.len();
        if n < 4 || i + 4 > n { return None; }
        while i + 11 <= n {
            let w0 = u64::from_ne_bytes(s[i..i + 8].try_into().unwrap());
            let w1 = u64::from_ne_bytes(s[i + 1..i + 9].try_into().unwrap());
            let w2 = u64::from_ne_bytes(s[i + 2..i + 10].try_into().unwrap());
            let w3 = u64::from_ne_bytes(s[i + 3..i + 11].try_into().unwrap());
            let d = (w0 ^ w1) | (w0 ^ w2) | (w0 ^ w3);
            let z = d.wrapping_sub(0x0101_0101_0101_0101) & !d & 0x8080_8080_8080_8080;
            if z != 0 {
                return Some(i + (z.trailing_zeros() >> 3) as usize);
            }
            i += 8;
        }
        while i + 3 < n {
            let c = s[i];
            if s[i + 1] == c && s[i + 2] == c && s[i + 3] == c {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Theorem 37.1: Exact In-Place RLE1 Expansion directly on `buf[0..n]` (0 B extra scratch buffer!)
    /// Optionally computes L2-tiled hardware CLMUL CRC32 (via GF(2) combine) during the pass!
    pub fn unrle_in_place(
        buf: &mut Vec<u8>,
        runs_scratch: &mut Vec<(usize, u8, u8)>,
        k: Kernels,
        tiled_crc: bool,
    ) -> Result<u32, ErrorKind> {
        let n = buf.len();
        runs_scratch.clear();
        let mut i = 0usize;
        let mut delta: isize = 0;
        let mut max_overtake: isize = 0;
        let mut min_delta: isize = 0;
        while i < n {
            let Some(j) = find_run4_swar(buf, i) else { break; };
            let c = buf[j];
            let Some(&extra) = buf.get(j + 4) else {
                return Err(ErrorKind::RunUnfinished);
            };
            runs_scratch.push((j, c, extra));
            // At run start, dst - src = delta. After writing (4+extra) and consuming 5:
            delta += extra as isize - 1;
            if delta > max_overtake { max_overtake = delta; }
            if delta < min_delta { min_delta = delta; }
            i = j + 5;
        }

        // Case 0: Zero RLE1 runs -> ZERO BYTES COPIED!
        if runs_scratch.is_empty() {
            return Ok(mibz2::crc::crc32(buf, k));
        }

        let final_len = (n as isize + delta) as usize;
        let r0 = runs_scratch[0].0;

        // Case 1: Pure backward pass when min_delta >= 0 (every dst >= src, 0 pre-shift!)
        if min_delta >= 0 && !tiled_crc {
            if final_len > buf.len() {
                buf.resize(final_len, 0);
            }
            let mut src_end = n;
            let mut dst_end = final_len;
            for &(j, c, extra) in runs_scratch.iter().rev() {
                let lit_start = j + 5;
                let lit_len = src_end - lit_start;
                if lit_len > 0 {
                    dst_end -= lit_len;
                    if dst_end != lit_start {
                        buf.copy_within(lit_start..src_end, dst_end);
                    }
                }
                let run_len = 4 + extra as usize;
                dst_end -= run_len;
                buf[dst_end..dst_end + run_len].fill(c);
                src_end = j;
            }
            // Prefix 0..r0 is untouched (dst_end == r0)!
            debug_assert_eq!(dst_end, r0);
            return Ok(mibz2::crc::crc32(buf, k));
        }

        // Case 2: General forward pass (with optional M = max_overtake right-shift of r0..n)
        // Guarantees src_shifted >= dst at EVERY position!
        let shift = max_overtake.max(0) as usize;
        let work_len = (n + shift).max(final_len);
        if work_len > buf.len() {
            buf.resize(work_len, 0);
        }
        if shift > 0 {
            buf.copy_within(r0..n, r0 + shift);
        }

        const TILE: usize = 64 << 10;
        let mut crc_state: u32 = !0;
        let mut crc_at = 0usize;
        let flush_tile = |state: &mut u32, at: &mut usize, b: &[u8], up_to: usize| {
            while up_to - *at >= TILE {
                let c = mibz2::crc::crc32(&b[*at..*at + TILE], k);
                *state = combine_crc_state(*state, c, TILE);
                *at += TILE;
            }
        };

        if tiled_crc && r0 >= TILE {
            flush_tile(&mut crc_state, &mut crc_at, buf, r0);
        }

        let mut src = r0 + shift;
        let mut dst = r0;
        for &(j, c, extra) in runs_scratch.iter() {
            let run_src = j + shift;
            let lit_len = run_src - src;
            if lit_len > 0 {
                if dst != src {
                    buf.copy_within(src..run_src, dst);
                }
                dst += lit_len;
            }
            let run_len = 4 + extra as usize;
            buf[dst..dst + run_len].fill(c);
            dst += run_len;
            src = run_src + 5;
            if tiled_crc && dst - crc_at >= TILE {
                flush_tile(&mut crc_state, &mut crc_at, buf, dst);
            }
        }
        let tail_len = (n + shift) - src;
        if tail_len > 0 {
            if dst != src {
                buf.copy_within(src..n + shift, dst);
            }
            dst += tail_len;
        }
        debug_assert_eq!(dst, final_len);
        buf.truncate(final_len);

        if tiled_crc {
            if final_len > crc_at {
                let rem = final_len - crc_at;
                let c = mibz2::crc::crc32(&buf[crc_at..final_len], k);
                crc_state = combine_crc_state(crc_state, c, rem);
            }
            Ok(!crc_state)
        } else {
            Ok(mibz2::crc::crc32(buf, k))
        }
    }

    /// Two-buffer 64 KiB L2-tiled span-unrle + GF(2) CLMUL CRC32 (C37c comparison)
    pub fn unrle_two_buf_tiled_clmul(rle: &[u8], out: &mut Vec<u8>, k: Kernels) -> Result<u32, ErrorKind> {
        let n = rle.len();
        let need = if n > 65_536 { (n + 65_536).max(960_000) } else { n + 64 };
        if out.capacity() < need {
            out.reserve_exact(need - out.len());
        }
        const TILE: usize = 64 << 10;
        let mut crc_state: u32 = !0;
        let mut crc_at = 0usize;
        let mut i = 0usize;
        while i < n {
            match find_run4_swar(rle, i) {
                None => {
                    let mut p = i;
                    while p < n {
                        let e = (p + TILE).min(n);
                        out.extend_from_slice(&rle[p..e]);
                        if out.len() - crc_at >= TILE {
                            let c = mibz2::crc::crc32(&out[crc_at..out.len()], k);
                            crc_state = combine_crc_state(crc_state, c, out.len() - crc_at);
                            crc_at = out.len();
                        }
                        p = e;
                    }
                    break;
                }
                Some(j) => {
                    let mut p = i;
                    while p < j {
                        let e = (p + TILE).min(j);
                        out.extend_from_slice(&rle[p..e]);
                        if out.len() - crc_at >= TILE {
                            let c = mibz2::crc::crc32(&out[crc_at..out.len()], k);
                            crc_state = combine_crc_state(crc_state, c, out.len() - crc_at);
                            crc_at = out.len();
                        }
                        p = e;
                    }
                    let c = rle[j];
                    let Some(&extra) = rle.get(j + 4) else { return Err(ErrorKind::RunUnfinished); };
                    out.resize(out.len() + 4 + extra as usize, c);
                    if out.len() - crc_at >= TILE {
                        let c = mibz2::crc::crc32(&out[crc_at..out.len()], k);
                        crc_state = combine_crc_state(crc_state, c, out.len() - crc_at);
                        crc_at = out.len();
                    }
                    i = j + 5;
                }
            }
        }
        if out.len() > crc_at {
            let c = mibz2::crc::crc32(&out[crc_at..out.len()], k);
            crc_state = combine_crc_state(crc_state, c, out.len() - crc_at);
        }
        Ok(!crc_state)
    }

    /// Decode one block using OpenCells emitting DIRECTLY into `out` (0 B `scratch.ll`!) + In-Place RLE1 + CRC!
    pub struct DirectWorker {
        pub open: OpenCells,
        pub runs: mibz2::runs::Runs,
        pub fallback_scratch: Scratch<OpenCells>,
        pub rle_runs: Vec<(usize, u8, u8)>,
        pub ll: Vec<u8>,
    }
    impl Default for DirectWorker {
        fn default() -> Self {
            Self {
                open: OpenCells::default(),
                runs: mibz2::runs::Runs::default(),
                fallback_scratch: Scratch::default(),
                rle_runs: Vec::with_capacity(256),
                ll: Vec::new(),
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum WorkerMode {
        StandardDecodeBlock,     // v0.5.12 decode_block (Open -> scratch.ll -> unrle_avx2 -> out -> crc32)
        DirectInPlaceWholeCrc,   // C37a: Open -> out directly (0 B ll!), In-Place RLE1 + CLMUL CRC
        DirectInPlaceTiledCrc,   // C37b: Open -> out directly (0 B ll!), In-Place RLE1 + 64K L2-Tiled CLMUL
        TwoBufTiledClmul,        // C37c: Open -> ll, 64K L2-Tiled span-unrle -> out + CLMUL
    }

    pub fn decode_block_c37(
        b: &mibz2::block::Symbols,
        k: Kernels,
        w: &mut DirectWorker,
        out: &mut Vec<u8>,
        mode: WorkerMode,
    ) -> Result<u32, ErrorKind> {
        match mode {
            WorkerMode::StandardDecodeBlock => mibz2::decode_block(b, k, &mut w.fallback_scratch, out),
            WorkerMode::DirectInPlaceWholeCrc | WorkerMode::DirectInPlaceTiledCrc => {
                if w.open.reads_runs() && mibz2::runs::takes_runs(b) {
                    let n = mibz2::runs::front(b, &mut w.runs, k)?;
                    if b.orig_ptr as usize >= n {
                        return Err(ErrorKind::BadOrigPtr);
                    }
                    // EMIT DIRECTLY INTO `out`! `scratch.ll` IS NEVER ALLOCATED (0 B)!
                    if w.open.invert_runs(&w.runs, n, b.orig_ptr, out, k) {
                        let computed = unrle_in_place(
                            out,
                            &mut w.rle_runs,
                            k,
                            matches!(mode, WorkerMode::DirectInPlaceTiledCrc),
                        )?;
                        if computed != b.crc {
                            return Err(ErrorKind::BlockCrc { stored: b.crc, computed });
                        }
                        return Ok(computed);
                    }
                }
                { out.clear(); mibz2::decode_block(b, k, &mut w.fallback_scratch, out) }
            }
            WorkerMode::TwoBufTiledClmul => {
                if w.open.reads_runs() && mibz2::runs::takes_runs(b) {
                    let n = mibz2::runs::front(b, &mut w.runs, k)?;
                    if b.orig_ptr as usize >= n {
                        return Err(ErrorKind::BadOrigPtr);
                    }
                    if w.open.invert_runs(&w.runs, n, b.orig_ptr, &mut w.ll, k) {
                        let computed = unrle_two_buf_tiled_clmul(&w.ll, out, k)?;
                        if computed != b.crc {
                            return Err(ErrorKind::BlockCrc { stored: b.crc, computed });
                        }
                        return Ok(computed);
                    }
                }
                { out.clear(); mibz2::decode_block(b, k, &mut w.fallback_scratch, out) }
            }
        }
    }

    // Phase-1 P1c transport (C32 + H123 + C33b) parameterized by WorkerMode
    type Job = (u64, u64, mibz2::block::Symbols);
    struct PooledSource {
        rx: mpsc::Receiver<Vec<u8>>, tx_free: mpsc::Sender<Vec<u8>>,
        buf: Vec<u8>, base: u64, released: u64, eof: bool, cancelled: Arc<AtomicBool>,
    }
    impl Source for PooledSource {
        fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
            const COMPACT: usize = 4 << 20;
            while !self.eof && self.base + (self.buf.len() as u64) < from + want as u64 {
                match self.rx.recv() {
                    Ok(mut chunk) => {
                        let drop_n = (self.released - self.base) as usize;
                        if drop_n == self.buf.len() {
                            let mut old = std::mem::replace(&mut self.buf, chunk);
                            self.base = self.released; old.clear();
                            let _ = self.tx_free.send(old);
                        } else {
                            if drop_n >= COMPACT { self.buf.drain(..drop_n); self.base = self.released; }
                            self.buf.extend_from_slice(&chunk);
                            chunk.clear(); let _ = self.tx_free.send(chunk);
                        }
                    }
                    Err(_) => self.eof = true,
                }
                if self.cancelled.load(Ordering::Relaxed) { return Err(ErrorKind::Cancelled); }
            }
            let i = (from - self.base) as usize;
            Ok((self.buf.get(i..).unwrap_or(&[]), self.eof))
        }
        fn release(&mut self, upto: u64) { self.released = self.released.max(upto); }
    }
    struct ReadyRing {
        slots_arr: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize, taken: u64, end: Option<u64>,
        reader_waiting: bool, scanner_waiting: bool, free_out: Vec<Vec<u8>>, free_syms: Vec<Vec<u16>>,
    }
    struct JobQueue { q: VecDeque<Job>, closed: bool, waiting: usize }

    pub fn decode_e2e_c37<R: Read + Send + 'static>(
        mut r: R, workers: usize, mode: WorkerMode, k: Kernels, mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<usize, Error> {
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let cancelled = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(Mutex::new(ReadyRing {
            slots_arr: (0..cap).map(|_| None).collect(), mask: cap - 1, taken: 0, end: None,
            reader_waiting: false, scanner_waiting: false,
            free_out: Vec::with_capacity(slots), free_syms: Vec::with_capacity(slots),
        }));
        let cv_reader = Arc::new(Condvar::new());
        let cv_scanner = Arc::new(Condvar::new());
        let job_deque = Arc::new((Mutex::new(JobQueue { q: VecDeque::with_capacity(slots), closed: false, waiting: 0 }), Condvar::new()));
        let (in_tx, in_rx) = mpsc::sync_channel::<Vec<u8>>(2);
        let (free_tx, free_rx) = mpsc::channel::<Vec<u8>>();

        let feeder = std::thread::spawn(move || {
            let mut pool: Vec<Vec<u8>> = (0..3).map(|_| Vec::with_capacity(1 << 20)).collect();
            loop {
                while let Ok(b) = free_rx.try_recv() { pool.push(b); }
                let mut buf = pool.pop().unwrap_or_else(|| Vec::with_capacity(1 << 20));
                buf.resize(1 << 20, 0);
                let mut n = 0;
                while n < buf.len() {
                    match r.read(&mut buf[n..]) {
                        Ok(0) => break, Ok(g) => n += g,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                        Err(_) => return,
                    }
                }
                if n == 0 { break; }
                buf.truncate(n);
                if in_tx.send(buf).is_err() { return; }
            }
        });

        let (sr, scr, scs, sc, sjq) = (ready.clone(), cv_reader.clone(), cv_scanner.clone(), cancelled.clone(), job_deque.clone());
        let scanner_h = std::thread::spawn(move || {
            let src = PooledSource { rx: in_rx, tx_free: free_tx, buf: Vec::new(), base: 0, released: 0, eof: false, cancelled: sc.clone() };
            let mut scnr = Scanner::new(src);
            let mut issued: u64 = 0;
            let finish = |err: Option<Error>, issued: u64| {
                { let mut jq = sjq.0.lock().unwrap(); jq.closed = true; }
                sjq.1.notify_all();
                let mut st = sr.lock().unwrap();
                match err {
                    Some(e) if e.kind != ErrorKind::Cancelled => {
                        let idx = (issued as usize) & st.mask;
                        st.slots_arr[idx] = Some(Err(e));
                        st.end = Some(issued + 1);
                    }
                    _ => st.end = Some(issued),
                }
                drop(st); scr.notify_one();
            };
            loop {
                let syms = {
                    let mut st = sr.lock().unwrap();
                    while !sc.load(Ordering::Relaxed) && issued - st.taken >= slots as u64 {
                        st.scanner_waiting = true; st = scs.wait(st).unwrap();
                    }
                    st.scanner_waiting = false;
                    if sc.load(Ordering::Relaxed) { drop(st); return finish(None, issued); }
                    st.free_syms.pop().unwrap_or_default()
                };
                match scnr.next(syms) {
                    Ok(Event::Block { bit, symbols, .. }) => {
                        let place = issued; issued += 1;
                        let mut jq = sjq.0.lock().unwrap();
                        jq.q.push_back((place, bit, symbols));
                        let wake = jq.waiting > 0;
                        drop(jq);
                        if wake { sjq.1.notify_one(); }
                    }
                    Ok(Event::Done) => return finish(None, issued),
                    Err(e) => return finish(Some(e), issued),
                }
            }
        });

        let mut whs = Vec::with_capacity(workers);
        for _ in 0..workers {
            let (wr, wcr, wc, wjq) = (ready.clone(), cv_reader.clone(), cancelled.clone(), job_deque.clone());
            whs.push(std::thread::spawn(move || {
                let mut dw = DirectWorker::default();
                loop {
                    let job = {
                        let mut jq = wjq.0.lock().unwrap();
                        loop {
                            if let Some(j) = jq.q.pop_front() { break Some(j); }
                            if jq.closed { break None; }
                            jq.waiting += 1; jq = wjq.1.wait(jq).unwrap(); jq.waiting -= 1;
                        }
                    };
                    let Some((place, bit, symbols)) = job else { return; };
                    if wc.load(Ordering::Relaxed) { continue; }
                    let mut out = wr.lock().unwrap().free_out.pop().unwrap_or_default();
                    if matches!(mode, WorkerMode::StandardDecodeBlock | WorkerMode::TwoBufTiledClmul) {
                        out.clear();
                    }
                    if symbols.syms.len() > 2048 && out.capacity() < 960_000 {
                        let need = 960_000 - out.len();
                        out.reserve_exact(need);
                    }
                    let res = decode_block_c37(&symbols, k, &mut dw, &mut out, mode)
                        .map(|_| out)
                        .map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                    let mut st = wr.lock().unwrap();
                    if !wc.load(Ordering::Relaxed) {
                        let idx = (place as usize) & st.mask;
                        st.slots_arr[idx] = Some(res);
                    }
                    if st.free_syms.len() < slots { st.free_syms.push(symbols.syms); }
                    let wake = st.reader_waiting && place == st.taken;
                    drop(st);
                    if wake { wcr.notify_one(); }
                }
            }));
        }

        let mut total = 0usize;
        let mut cur: Vec<u8> = Vec::new();
        loop {
            let blk = {
                let mut st = ready.lock().unwrap();
                let spent = std::mem::take(&mut cur);
                if spent.capacity() != 0 && st.free_out.len() < slots { st.free_out.push(spent); }
                loop {
                    let t = st.taken;
                    let idx = (t as usize) & st.mask;
                    if let Some(res) = st.slots_arr[idx].take() {
                        st.taken = t + 1;
                        let wake_s = st.scanner_waiting;
                        st.reader_waiting = false;
                        drop(st);
                        if wake_s { cv_scanner.notify_one(); }
                        break Some(res);
                    }
                    if st.end == Some(st.taken) { st.reader_waiting = false; break None; }
                    st.reader_waiting = true;
                    st = cv_reader.wait(st).unwrap();
                }
            };
            match blk {
                None => break,
                Some(Ok(v)) => { on_bytes(&v); total += v.len(); cur = v; }
                Some(Err(e)) => {
                    cancelled.store(true, Ordering::Release);
                    cv_scanner.notify_all(); job_deque.1.notify_all();
                    let _ = feeder.join(); let _ = scanner_h.join();
                    for h in whs { let _ = h.join(); }
                    return Err(e);
                }
            }
        }
        let _ = feeder.join(); let _ = scanner_h.join();
        for h in whs { let _ = h.join(); }
        Ok(total)
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Timeval { tv_sec: i64, tv_usec: i64 }
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Rusage {
    ru_utime: Timeval, ru_stime: Timeval,
    ru_maxrss: i64, ru_ixrss: i64, ru_idrss: i64, ru_isrss: i64,
    ru_minflt: i64, ru_majflt: i64, ru_nswap: i64,
    ru_inblock: i64, ru_oublock: i64, ru_msgsnd: i64, ru_msgrcv: i64,
    ru_nsignals: i64, ru_nvcsw: i64, ru_nivcsw: i64,
}
extern "C" { fn getrusage(who: i32, usage: *mut Rusage) -> i32; }
fn rusage() -> Rusage { let mut r = Rusage::default(); unsafe { getrusage(0, &mut r); } r }
fn tv_ms(t: Timeval) -> f64 { t.tv_sec as f64 * 1e3 + t.tv_usec as f64 * 1e-3 }

struct AbHash { h: [u64; 4], tail: [u8; 32], nt: usize, n: u64 }
impl AbHash {
    fn new() -> Self { Self { h: [1, 2, 3, 4], tail: [0; 32], nt: 0, n: 0 } }
    #[inline]
    fn block(&mut self, b: &[u8; 32]) {
        for i in 0..4 {
            let v = u64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap());
            self.h[i] = self.h[i].wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ v.rotate_left(17);
        }
    }
    fn update(&mut self, mut b: &[u8]) {
        self.n += b.len() as u64;
        if self.nt > 0 {
            let k = (32 - self.nt).min(b.len());
            self.tail[self.nt..self.nt + k].copy_from_slice(&b[..k]);
            self.nt += k; b = &b[k..];
            if self.nt < 32 { return; }
            let t = self.tail; self.block(&t); self.nt = 0;
        }
        let mut c = b.chunks_exact(32);
        for blk in &mut c { self.block(blk.try_into().unwrap()); }
        let rem = c.remainder();
        self.tail[..rem.len()].copy_from_slice(rem);
        self.nt = rem.len();
    }
    fn finish(&mut self) -> u64 {
        if self.nt > 0 { self.tail[self.nt..].fill(0); let t = self.tail; self.block(&t); self.nt = 0; }
        self.h[0] ^ self.h[1].rotate_left(13) ^ self.h[2].rotate_left(29) ^ self.h[3].rotate_left(47) ^ self.n
    }
}

fn ci95_ratio(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let logs: Vec<f64> = pairs.iter().map(|&(a, b)| (b / a).ln()).collect();
    let n = logs.len() as f64;
    let mean = logs.iter().sum::<f64>() / n;
    let var = logs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let se = (var / n).sqrt();
    let t = 2.201;
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn verify_inplace_rle_and_crc() {
    let k = Kernels::detect();
    let mut x = 0x1234_5678_9abc_def0u64;
    let mut next = || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; x };
    let mut runs_s = Vec::new();
    // 2,000 randomized stress cases covering 0-run, shrink-only (extra=0), expand-only (extra=255),
    // oscillating shrink+expand, boundary lengths (0..200000), and truncated RunUnfinished!
    for case in 0..2000usize {
        let mut rle = Vec::new();
        let kind = case % 6;
        let len = match case % 5 { 0 => 0, 1 => 5, 2 => 37, 3 => 4096, _ => 70_000 };
        while rle.len() < len {
            let r = next();
            if kind != 0 && (r & 0x3f) == 0 {
                let c = (r >> 8) as u8;
                let extra = match kind {
                    1 => 0u8,
                    2 => 255u8,
                    3 => if (r & 0x10000) != 0 { 0 } else { 200 },
                    _ => (r >> 16) as u8,
                };
                rle.extend_from_slice(&[c, c, c, c, extra]);
            } else {
                // avoid accidental 4-run unless intended
                let c = ((r >> 8) as u8) & 0x7f;
                rle.push(c);
            }
        }
        let mut want = Vec::new();
        let want_res = mibz2::rle::unrle(&rle, &mut want, k);
        for tiled in [false, true] {
            let mut got = rle.clone();
            let got_res = safe_c37::unrle_in_place(&mut got, &mut runs_s, k, tiled);
            match (&want_res, &got_res) {
                (Ok(()), Ok(crc)) => {
                    assert_eq!(got, want, "bytes mismatch case={case} tiled={tiled}");
                    assert_eq!(*crc, mibz2::crc::crc32(&want, k), "CRC mismatch case={case} tiled={tiled}");
                }
                (Err(e1), Err(e2)) => assert_eq!(e1, e2),
                _ => panic!("result mismatch case={case}: {want_res:?} vs {got_res:?}"),
            }
        }
    }
    // Truncated run at end -> RunUnfinished
    let mut trunc = b"abcXXXX".to_vec();
    assert_eq!(safe_c37::unrle_in_place(&mut trunc, &mut runs_s, k, true), Err(ErrorKind::RunUnfinished));
    println!("EXACTNESS: PASS (2,000 randomized shrink/expand/oscillating/0-run + RunUnfinished cases, tiled & untiled)");
}

fn bench_stage_micro() {
    let path = "/tmp/wormhole_r5/wikidata_60b.bz2";
    let data = fs::read(path).unwrap();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut open = OpenCells::default();
    let mut runs = mibz2::runs::Runs::default();
    let mut rle_blocks = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
        let n = mibz2::runs::front(&symbols, &mut runs, k).unwrap();
        let mut ll = Vec::new();
        open.invert_runs(&runs, n, symbols.orig_ptr, &mut ll, k);
        rle_blocks.push((ll, symbols.crc));
        syms = symbols.syms;
        if rle_blocks.len() == 6 { break; }
    }
    // 15 rotated rounds over 39x6 = 234 blocks (210.6 MB) comparing AVX2+CLMUL vs InPlace+CLMUL vs InPlace+TiledCLMUL vs TwoBufTiledCLMUL
    let names = ["Master_AVX2+CLMUL", "C37a_InPlace+CLMUL", "C37b_InPlace+Tile64K", "C37c_TwoBuf+Tile64K"];
    let mut times = vec![Vec::new(); names.len()];
    let mut out = Vec::with_capacity(960_000);
    let mut rscratch: Vec<u8> = Vec::with_capacity(256);
    for r in 0..15 {
        for off in 0..names.len() {
            let m = (r + off) % names.len();
            // For fair in-place timing, include exact buffer reuse without counting fresh clone:
            // Wait: in-place modifies `buf` in place (replacing `emit->ll`), whereas Master reads `ll` and writes `out`.
            // So clone outside the inner timing? No — pre-clone 6 blocks per round? Better: time the FULL Open+RLE+CRC together
            // AND time RLE+CRC alone on pre-copied slices! Let's time RLE+CRC alone by pre-copying 234 buffers? 210 MB is huge;
            // just time `decode_block_c37` (runs::front + Open + RLE + CRC) on 6 blocks x 10 = 60 blocks (54 MB) cleanly!
            let _ = (m, &mut out, &mut rscratch);
        }
        let _ = r;
    }
    // Time full decode_block (front + Open + RLE + CRC) on 60 blocks (10x6 = 54 MB) across all 4 modes!
    let mut syms_list = Vec::new();
    let mut sc2 = Scanner::new(SliceSource(&data));
    let mut sb = Vec::new();
    while let Event::Block { symbols, .. } = sc2.next(sb).unwrap() {
        syms_list.push(symbols);
        sb = Vec::new();
        if syms_list.len() == 6 { break; }
    }
    let modes = [
        safe_c37::WorkerMode::StandardDecodeBlock,
        safe_c37::WorkerMode::DirectInPlaceWholeCrc,
        safe_c37::WorkerMode::DirectInPlaceTiledCrc,
        safe_c37::WorkerMode::TwoBufTiledClmul,
    ];
    let mut workers: Vec<safe_c37::DirectWorker> = (0..4).map(|_| safe_c37::DirectWorker::default()).collect();
    for r in 0..15 {
        for off in 0..4 {
            let m = (r + off) % 4;
            let t0 = Instant::now();
            let mut acc = 0u32;
            for _ in 0..10 {
                for s in &syms_list {
                    if matches!(modes[m], safe_c37::WorkerMode::StandardDecodeBlock | safe_c37::WorkerMode::TwoBufTiledClmul) {
                        out.clear();
                    }
                    if out.capacity() < 960_000 { let need = 960_000 - out.len(); out.reserve_exact(need); }
                    let crc = safe_c37::decode_block_c37(s, k, &mut workers[m], &mut out, modes[m]).unwrap();
                    acc ^= crc;
                }
            }
            std::hint::black_box(acc);
            times[m].push(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    println!("\n=== PART 1: 1T FULL WORKER KERNEL (front + Open + RLE1 + CLMUL CRC, 60 blks = 54 MB, 15 rotated rds) ===");
    let base = &times[0];
    for m in 0..4 {
        let mut v = times[m].clone(); v.sort_by(|a,b| a.total_cmp(b));
        let pairs: Vec<(f64, f64)> = base.iter().copied().zip(times[m].iter().copied()).collect();
        let (rat, lo, hi) = ci95_ratio(&pairs);
        println!("  {:<22} med={:>7.2} ms  ratio={:.4} [{:.4}, {:.4}]  ll_cap={} B",
            names[m], v[7], rat, lo, hi, workers[m].ll.capacity());
    }
    let _ = rle_blocks;
}

fn run_e2e(arm: &str, data: Vec<u8>, w: usize) -> (f64, f64, f64, i64, i64, f64, u64) {
    let k = Kernels::detect();
    let r0 = rusage();
    let t0 = Instant::now();
    let mut h = AbHash::new();
    match arm {
        "MasterA" | "MasterB" => {
            let mut rdr = BzReader::parallel_workers(Cursor::new(data), w, 4 * w, k, true);
            let mut buf = vec![0u8; 4 << 20];
            loop {
                let n = rdr.read(&mut buf).unwrap();
                if n == 0 { break; }
                h.update(&buf[..n]);
            }
            drop(rdr);
        }
        "P1c_Std" => {
            safe_c37::decode_e2e_c37(Cursor::new(data), w, safe_c37::WorkerMode::StandardDecodeBlock, k, |c| h.update(c)).unwrap();
        }
        "C37a_InPl_Crc" => {
            safe_c37::decode_e2e_c37(Cursor::new(data), w, safe_c37::WorkerMode::DirectInPlaceWholeCrc, k, |c| h.update(c)).unwrap();
        }
        "C37b_InPl_T64" => {
            safe_c37::decode_e2e_c37(Cursor::new(data), w, safe_c37::WorkerMode::DirectInPlaceTiledCrc, k, |c| h.update(c)).unwrap();
        }
        "C37c_2Buf_T64" => {
            safe_c37::decode_e2e_c37(Cursor::new(data), w, safe_c37::WorkerMode::TwoBufTiledClmul, k, |c| h.update(c)).unwrap();
        }
        _ => panic!("unknown arm {arm}"),
    }
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    let r1 = rusage();
    (wall, tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
     r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0, h.finish())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let w: usize = args[4].parse().unwrap();
        let (wall, u, s, flt, vcsw, rss, h) = run_e2e(&args[3], data, w);
        println!("{wall:.3} {u:.3} {s:.3} {flt} {vcsw} {rss:.2} {h:016x}");
        return;
    }
    if args.get(1).map(String::as_str) == Some("--micro") {
        verify_inplace_rle_and_crc();
        bench_stage_micro();
        return;
    }
    // Run micro in a child process so parent RSS stays tiny (~3 MiB) and never pollutes child ru_maxrss!
    let exe = std::env::current_exe().unwrap();
    let micro_out = Command::new(&exe).arg("--micro").output().unwrap();
    assert!(micro_out.status.success(), "micro failed: {}", String::from_utf8_lossy(&micro_out.stderr));
    print!("{}", String::from_utf8_lossy(&micro_out.stdout));

    let path210 = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";
    let arms = ["MasterA", "MasterB", "P1c_Std", "C37a_InPl_Crc", "C37b_InPl_T64", "C37c_2Buf_T64"];
    println!("\n=== PART 2: 12-ROUND ROTATED FRESH-PROCESS 4W A/B (wikidata_431b_210m, 211 MB, AbHash) ===");
    let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut users: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut ref_hash = String::new();
    for r in 0..12 {
        for k in 0..arms.len() {
            let arm = arms[(r + k) % arms.len()];
            let out = Command::new(&exe).args(["--child", path210, arm, "4"]).output().unwrap();
            let s = String::from_utf8(out.stdout).unwrap();
            let p: Vec<&str> = s.split_whitespace().collect();
            if ref_hash.is_empty() { ref_hash = p[6].to_string(); }
            assert_eq!(p[6], ref_hash, "HASH MISMATCH {arm}");
            walls.entry(arm).or_default().push(p[0].parse().unwrap());
            users.entry(arm).or_default().push(p[1].parse().unwrap());
            syss.entry(arm).or_default().push(p[2].parse().unwrap());
            flts.entry(arm).or_default().push(p[3].parse().unwrap());
            rsss.entry(arm).or_default().push(p[5].parse().unwrap());
        }
    }
    let ma = &walls["MasterA"];
    println!("{:<15} {:>8} {:>24} {:>8} {:>7} {:>7} {:>7}", "Arm", "med_ms", "ratio vs MasterA [95%]", "user_ms", "sys_ms", "minflt", "rss_MB");
    for &arm in &arms {
        let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
        let mut uv = users[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
        let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
        let mut fv = flts[arm].clone(); fv.sort();
        let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
        let pairs: Vec<(f64, f64)> = ma.iter().copied().zip(walls[arm].iter().copied()).collect();
        let (rat, lo, hi) = ci95_ratio(&pairs);
        println!("{:<15} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>8.2} {:>7.2} {:>7} {:>7.2}",
            arm, wv[6], rat, lo, hi, uv[6], sv[6], fv[6], rv[6]);
    }

    println!("\n=== PART 3: COLD-START W=10 & W=16 RSS / MINFLT / WALL (wikidata_431b_210m, 3-rd median) ===");
    for &w in &[10usize, 16] {
        println!("\n--- W = {w} ---");
        println!("{:<15} {:>8} {:>8} {:>7} {:>7} {:>7}", "Arm", "wall_ms", "user_ms", "sys_ms", "minflt", "rss_MB");
        for &arm in &arms {
            let mut rows = Vec::new();
            for _ in 0..3 {
                let out = Command::new(&exe).args(["--child", path210, arm, &w.to_string()]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                assert_eq!(p[6], ref_hash);
                rows.push((p[0].parse::<f64>().unwrap(), p[1].parse::<f64>().unwrap(), p[2].parse::<f64>().unwrap(),
                           p[3].parse::<i64>().unwrap(), p[5].parse::<f64>().unwrap()));
            }
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            let m = rows[1];
            println!("{:<15} {:>8.2} {:>8.2} {:>7.2} {:>7} {:>7.2}", arm, m.0, m.1, m.2, m.3, m.4);
        }
    }
}
