use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Command;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BzReader, Error, ErrorKind, Kernels, Scratch};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, SliceSource, Source};

// ---- Cycle 33: Safe-Rust RLE1 + CRC32 Fusion & Output Assembly (Batch-Fill & BufRead Zero-Copy) ----
mod safe_c33 {
    #![forbid(unsafe_code)]
    use super::*;

    pub const POLY: u32 = 0x04C1_1DB7;

    const fn make_tables() -> [[u32; 256]; 16] {
        let mut t = [[0u32; 256]; 16];
        let mut i = 0;
        while i < 256 {
            let mut c = (i as u32) << 24;
            let mut k = 0;
            while k < 8 {
                c = if c & 0x8000_0000 != 0 { (c << 1) ^ POLY } else { c << 1 };
                k += 1;
            }
            t[0][i] = c;
            i += 1;
        }
        let mut j = 1;
        while j < 16 {
            let mut i = 0;
            while i < 256 {
                let p = t[j - 1][i];
                t[j][i] = (p << 8) ^ t[0][(p >> 24) as usize];
                i += 1;
            }
            j += 1;
        }
        t
    }

    static TABLES16: [[u32; 256]; 16] = make_tables();

    /// Safe-Rust slice-by-8 CRC32 update (exact mirror of src/crc.rs::update)
    #[inline]
    pub fn crc32_sb8(mut s: u32, data: &[u8]) -> u32 {
        let t = &TABLES16;
        let mut chunks = data.chunks_exact(8);
        for c in &mut chunks {
            let hi = s ^ u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
            s = t[7][(hi >> 24) as usize]
                ^ t[6][((hi >> 16) & 0xff) as usize]
                ^ t[5][((hi >> 8) & 0xff) as usize]
                ^ t[4][(hi & 0xff) as usize]
                ^ t[3][c[4] as usize]
                ^ t[2][c[5] as usize]
                ^ t[1][c[6] as usize]
                ^ t[0][c[7] as usize];
        }
        for &b in chunks.remainder() {
            s = (s << 8) ^ t[0][((s >> 24) as u8 ^ b) as usize];
        }
        s
    }

    /// Safe-Rust slice-by-16 CRC32 update (2x wider ILP across 16 tables = 16 KiB in L1d)
    #[inline]
    pub fn crc32_sb16(mut s: u32, data: &[u8]) -> u32 {
        let t = &TABLES16;
        let mut chunks = data.chunks_exact(16);
        for c in &mut chunks {
            let hi = s ^ u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
            s = t[15][(hi >> 24) as usize]
                ^ t[14][((hi >> 16) & 0xff) as usize]
                ^ t[13][((hi >> 8) & 0xff) as usize]
                ^ t[12][(hi & 0xff) as usize]
                ^ t[11][c[4] as usize]
                ^ t[10][c[5] as usize]
                ^ t[9][c[6] as usize]
                ^ t[8][c[7] as usize]
                ^ t[7][c[8] as usize]
                ^ t[6][c[9] as usize]
                ^ t[5][c[10] as usize]
                ^ t[4][c[11] as usize]
                ^ t[3][c[12] as usize]
                ^ t[2][c[13] as usize]
                ^ t[1][c[14] as usize]
                ^ t[0][c[15] as usize];
        }
        crc32_sb8(s, chunks.remainder())
    }

    /// Find index of first 4-equal-byte run in `s` starting at or after `from`, or `None`.
    /// Pure safe Rust SWAR: scans 8 bytes at a time checking if any byte equals its next 3 neighbors.
    #[inline]
    pub fn find_run4_swar(s: &[u8], mut i: usize) -> Option<usize> {
        let n = s.len();
        if n < 4 || i + 4 > n { return None; }
        // Fast 8-byte SWAR scan while i + 11 <= n
        while i + 11 <= n {
            let w0 = u64::from_ne_bytes(s[i..i + 8].try_into().unwrap());
            let w1 = u64::from_ne_bytes(s[i + 1..i + 9].try_into().unwrap());
            let w2 = u64::from_ne_bytes(s[i + 2..i + 10].try_into().unwrap());
            let w3 = u64::from_ne_bytes(s[i + 3..i + 11].try_into().unwrap());
            let d = (w0 ^ w1) | (w0 ^ w2) | (w0 ^ w3);
            // Any zero byte in `d` means 4 equal consecutive bytes starting at that lane!
            let z = d.wrapping_sub(0x0101_0101_0101_0101) & !d & 0x8080_8080_8080_8080;
            if z != 0 {
                let lane = (z.trailing_zeros() >> 3) as usize;
                return Some(i + lane);
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

    /// C1: Span-bulk-copy safe unrle (copies entire run-free spans in one memcpy, exact-fit reserve)
    pub fn unrle_span_safe(rle: &[u8], out: &mut Vec<u8>) -> Result<(), ErrorKind> {
        let n = rle.len();
        let need = if n > 65_536 { (n + 65_536).max(960_000) } else { n + 64 };
        if out.capacity() < need {
            out.reserve_exact(need - out.len());
        }
        let mut i = 0;
        while i < n {
            match find_run4_swar(rle, i) {
                None => {
                    out.extend_from_slice(&rle[i..]);
                    return Ok(());
                }
                Some(j) => {
                    if j > i {
                        out.extend_from_slice(&rle[i..j]);
                    }
                    let c = rle[j];
                    let Some(&extra) = rle.get(j + 4) else {
                        return Err(ErrorKind::RunUnfinished);
                    };
                    let total_run = 4 + extra as usize;
                    out.resize(out.len() + total_run, c);
                    i = j + 5;
                }
            }
        }
        Ok(())
    }

    /// C2: L1/L2-tiled fused unrle + CRC32 (updates CRC on every ~32 KiB tile while 100% L1/L2 hot!)
    pub fn unrle_and_crc_tiled(
        rle: &[u8],
        out: &mut Vec<u8>,
        tile_bytes: usize,
        use_sb16: bool,
    ) -> Result<u32, ErrorKind> {
        let n = rle.len();
        let start = out.len();
        let need = if n > 65_536 { (n + 65_536).max(960_000) } else { n + 64 };
        if out.capacity() < start + need {
            out.reserve_exact(start + need - out.len());
        }
        let mut i = 0;
        let mut crc_state: u32 = !0;
        let mut crc_at = start;

        let flush_crc = |state: &mut u32, at: &mut usize, buf: &[u8]| {
            if buf.len() > *at {
                *state = if use_sb16 {
                    crc32_sb16(*state, &buf[*at..])
                } else {
                    crc32_sb8(*state, &buf[*at..])
                };
                *at = buf.len();
            }
        };

        while i < n {
            match find_run4_swar(rle, i) {
                None => {
                    // Process remaining tail in L1/L2-sized tiles
                    let mut p = i;
                    while p < n {
                        let chunk_end = (p + tile_bytes).min(n);
                        out.extend_from_slice(&rle[p..chunk_end]);
                        if out.len() - crc_at >= tile_bytes {
                            flush_crc(&mut crc_state, &mut crc_at, out);
                        }
                        p = chunk_end;
                    }
                    break;
                }
                Some(j) => {
                    let mut p = i;
                    while p < j {
                        let chunk_end = (p + tile_bytes).min(j);
                        out.extend_from_slice(&rle[p..chunk_end]);
                        if out.len() - crc_at >= tile_bytes {
                            flush_crc(&mut crc_state, &mut crc_at, out);
                        }
                        p = chunk_end;
                    }
                    let c = rle[j];
                    let Some(&extra) = rle.get(j + 4) else {
                        return Err(ErrorKind::RunUnfinished);
                    };
                    out.resize(out.len() + 4 + extra as usize, c);
                    if out.len() - crc_at >= tile_bytes {
                        flush_crc(&mut crc_state, &mut crc_at, out);
                    }
                    i = j + 5;
                }
            }
        }
        flush_crc(&mut crc_state, &mut crc_at, out);
        Ok(!crc_state)
    }

    /// Output assembly modes for end-to-end 4W/10W A/B:
    #[derive(Clone, Copy, Debug)]
    pub enum ReadMode {
        OneBlockCopy,   // master behavior: 1 block per read() call + copy into 4 MiB buf
        BatchFillCopy,  // C4a: fill caller 4 MiB buf across multiple ready blocks per lock
        ZeroCopyBufRead,// C4b: BufRead::fill_buf / consume — zero memcpy!
    }

    struct State {
        ready: BTreeMap<u64, Result<Vec<u8>, Error>>,
        issued: u64,
        taken: u64,
        end: Option<u64>,
        cancelled: bool,
        free_out: Vec<Vec<u8>>,
        free_syms: Vec<Vec<u16>>,
    }
    struct Shared {
        state: Mutex<State>,
        cv: Condvar,
        slots: u64,
    }
    struct PooledSource<'a> {
        rx: mpsc::Receiver<Vec<u8>>,
        tx_free: mpsc::Sender<Vec<u8>>,
        buf: Vec<u8>,
        base: u64,
        released: u64,
        eof: bool,
        shared: &'a Shared,
    }
    impl Source for PooledSource<'_> {
        fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
            const COMPACT: usize = 4 << 20;
            while !self.eof && self.base + (self.buf.len() as u64) < from + want as u64 {
                match self.rx.recv() {
                    Ok(mut chunk) => {
                        let drop_n = (self.released - self.base) as usize;
                        if drop_n == self.buf.len() {
                            let mut old = std::mem::replace(&mut self.buf, chunk);
                            self.base = self.released;
                            old.clear();
                            let _ = self.tx_free.send(old);
                        } else {
                            if drop_n >= COMPACT {
                                self.buf.drain(..drop_n);
                                self.base = self.released;
                            }
                            self.buf.extend_from_slice(&chunk);
                            chunk.clear();
                            let _ = self.tx_free.send(chunk);
                        }
                    }
                    Err(_) => self.eof = true,
                }
                if self.shared.state.lock().unwrap().cancelled {
                    return Err(ErrorKind::Cancelled);
                }
            }
            let i = (from - self.base) as usize;
            Ok((self.buf.get(i..).unwrap_or(&[]), self.eof))
        }
        fn release(&mut self, upto: u64) { self.released = self.released.max(upto); }
    }

    pub fn decode_c33_e2e<R: Read + Send + 'static>(
        mut r: R,
        workers: usize,
        mode: ReadMode,
        k: Kernels,
        mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<(usize, usize), Error> {
        let slots = (4 * workers).max(workers) as u64;
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                ready: BTreeMap::new(),
                issued: 0,
                taken: 0,
                end: None,
                cancelled: false,
                free_out: Vec::with_capacity(slots as usize),
                free_syms: Vec::with_capacity(slots as usize),
            }),
            cv: Condvar::new(),
            slots,
        });
        let (in_tx, in_rx) = mpsc::sync_channel::<Vec<u8>>(2);
        let (free_tx, free_rx) = mpsc::channel::<Vec<u8>>();
        let (job_tx, job_rx) = mpsc::channel::<(u64, u64, mibz2::block::Symbols)>();
        let job_rx = Arc::new(Mutex::new(job_rx));

        let feeder = std::thread::spawn(move || {
            let mut pool: Vec<Vec<u8>> = (0..3).map(|_| Vec::with_capacity(1 << 20)).collect();
            loop {
                while let Ok(b) = free_rx.try_recv() { pool.push(b); }
                let mut buf = pool.pop().unwrap_or_else(|| Vec::with_capacity(1 << 20));
                buf.resize(1 << 20, 0);
                let mut n = 0;
                while n < buf.len() {
                    match r.read(&mut buf[n..]) {
                        Ok(0) => break,
                        Ok(g) => n += g,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                        Err(_) => return,
                    }
                }
                if n == 0 { break; }
                buf.truncate(n);
                if in_tx.send(buf).is_err() { return; }
            }
        });

        let sh_s = shared.clone();
        let scanner_h = std::thread::spawn(move || {
            let src = PooledSource { rx: in_rx, tx_free: free_tx, buf: Vec::new(), base: 0, released: 0, eof: false, shared: &sh_s };
            let mut sc = Scanner::new(src);
            loop {
                let syms = {
                    let mut st = sh_s.state.lock().unwrap();
                    while !st.cancelled && st.issued - st.taken >= sh_s.slots {
                        st = sh_s.cv.wait(st).unwrap();
                    }
                    if st.cancelled {
                        st.end = Some(st.issued); drop(st); sh_s.cv.notify_all(); return;
                    }
                    st.free_syms.pop().unwrap_or_default()
                };
                match sc.next(syms) {
                    Ok(Event::Block { bit, symbols, .. }) => {
                        let mut st = sh_s.state.lock().unwrap();
                        let place = st.issued;
                        st.issued += 1;
                        drop(st);
                        if job_tx.send((place, bit, symbols)).is_err() {
                            let mut st = sh_s.state.lock().unwrap();
                            st.end = Some(st.issued); drop(st); sh_s.cv.notify_all(); return;
                        }
                    }
                    Ok(Event::Done) => {
                        let mut st = sh_s.state.lock().unwrap();
                        st.end = Some(st.issued); drop(st); sh_s.cv.notify_all(); return;
                    }
                    Err(e) => {
                        let mut st = sh_s.state.lock().unwrap();
                        let at = st.issued;
                        st.ready.insert(at, Err(e)); st.issued += 1; st.end = Some(at + 1);
                        drop(st); sh_s.cv.notify_all(); return;
                    }
                }
            }
        });

        let mut whs = Vec::with_capacity(workers);
        for _ in 0..workers {
            let sh_w = shared.clone();
            let rx_w = job_rx.clone();
            whs.push(std::thread::spawn(move || {
                let mut scratch = Scratch::<OpenCells>::default();
                loop {
                    let job = { rx_w.lock().unwrap().recv() };
                    let Ok((place, bit, symbols)) = job else { return; };
                    let mut out = {
                        let mut st = sh_w.state.lock().unwrap();
                        if st.cancelled { continue; }
                        st.free_out.pop().unwrap_or_default()
                    };
                    out.clear();
                    if symbols.syms.len() > 2048 && out.capacity() < 960_000 {
                        out.reserve_exact(960_000);
                    }
                    let res = mibz2::decode_block(&symbols, k, &mut scratch, &mut out)
                        .map(|_| out)
                        .map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                    let mut st = sh_w.state.lock().unwrap();
                    if !st.cancelled { st.ready.insert(place, res); }
                    if st.free_syms.len() < sh_w.slots as usize { st.free_syms.push(symbols.syms); }
                    drop(st);
                    sh_w.cv.notify_all();
                }
            }));
        }

        let mut total = 0usize;
        let mut lock_ops = 0usize;
        match mode {
            ReadMode::OneBlockCopy => {
                let mut read_buf = vec![0u8; 4 << 20];
                let mut cur = Vec::new();
                loop {
                    let blk = {
                        lock_ops += 1;
                        let mut st = shared.state.lock().unwrap();
                        let spent = std::mem::take(&mut cur);
                        if spent.capacity() != 0 && st.free_out.len() < shared.slots as usize {
                            st.free_out.push(spent);
                        }
                        loop {
                            let taken = st.taken;
                            if let Some(res) = st.ready.remove(&taken) {
                                st.taken += 1; shared.cv.notify_all(); break Some(res);
                            }
                            if st.end == Some(st.taken) { break None; }
                            st = shared.cv.wait(st).unwrap();
                        }
                    };
                    match blk {
                        None => break,
                        Some(Ok(v)) => {
                            let mut at = 0;
                            while at < v.len() {
                                let n = read_buf.len().min(v.len() - at);
                                read_buf[..n].copy_from_slice(&v[at..at + n]);
                                on_bytes(&read_buf[..n]);
                                at += n; total += n;
                            }
                            cur = v;
                        }
                        Some(Err(e)) => {
                            shared.state.lock().unwrap().cancelled = true;
                            shared.cv.notify_all();
                            let _ = feeder.join(); let _ = scanner_h.join();
                            for h in whs { let _ = h.join(); }
                            return Err(e);
                        }
                    }
                }
            }
            ReadMode::BatchFillCopy => {
                // C4a: Drain multiple already-ready consecutive blocks per lock into caller's 4 MiB buf,
                // returning spent buffers immediately!
                let mut read_buf = vec![0u8; 4 << 20];
                let mut cur: Vec<u8> = Vec::new();
                let mut cur_at: usize = 0;
                let mut batch: Vec<Vec<u8>> = Vec::with_capacity(8);
                let mut done = false;
                while !done {
                    let mut filled = 0usize;
                    if cur_at < cur.len() {
                        let n = (read_buf.len() - filled).min(cur.len() - cur_at);
                        read_buf[filled..filled + n].copy_from_slice(&cur[cur_at..cur_at + n]);
                        filled += n; cur_at += n;
                    }
                    if cur_at == cur.len() && filled < read_buf.len() {
                        // Pop a batch of ready blocks under ONE lock!
                        lock_ops += 1;
                        let mut st = shared.state.lock().unwrap();
                        if cur.capacity() != 0 && st.free_out.len() < shared.slots as usize {
                            st.free_out.push(std::mem::take(&mut cur));
                        }
                        cur_at = 0;
                        let mut popped_any = false;
                        loop {
                            let taken = st.taken;
                            if let Some(res) = st.ready.remove(&taken) {
                                st.taken += 1;
                                popped_any = true;
                                match res {
                                    Ok(v) => {
                                        let vlen = v.len();
                                        batch.push(v);
                                        if filled + batch.iter().map(|b| b.len()).sum::<usize>() >= read_buf.len() {
                                            break;
                                        }
                                        let _ = vlen;
                                    }
                                    Err(e) => {
                                        st.cancelled = true; drop(st); shared.cv.notify_all();
                                        let _ = feeder.join(); let _ = scanner_h.join();
                                        for h in whs { let _ = h.join(); }
                                        return Err(e);
                                    }
                                }
                            } else if popped_any || filled > 0 {
                                break;
                            } else if st.end == Some(st.taken) {
                                done = true;
                                break;
                            } else {
                                st = shared.cv.wait(st).unwrap();
                            }
                        }
                        if popped_any { shared.cv.notify_all(); }
                        drop(st);

                        // Copy batch into read_buf and return completed buffers immediately!
                        let mut spent_batch: Vec<Vec<u8>> = Vec::new();
                        for v in batch.drain(..) {
                            let rem = read_buf.len() - filled;
                            if v.len() <= rem {
                                read_buf[filled..filled + v.len()].copy_from_slice(&v);
                                filled += v.len();
                                spent_batch.push(v);
                            } else {
                                read_buf[filled..filled + rem].copy_from_slice(&v[..rem]);
                                filled += rem;
                                cur = v;
                                cur_at = rem;
                                break;
                            }
                        }
                        if !spent_batch.is_empty() {
                            let mut st = shared.state.lock().unwrap();
                            for s in spent_batch {
                                if s.capacity() != 0 && st.free_out.len() < shared.slots as usize {
                                    st.free_out.push(s);
                                }
                            }
                        }
                    }
                    if filled > 0 {
                        on_bytes(&read_buf[..filled]);
                        total += filled;
                    }
                }
            }
            ReadMode::ZeroCopyBufRead => {
                // C4b: BufRead::fill_buf + consume — ZERO memcpy! Caller reads directly from block Vec<u8>!
                let mut cur: Vec<u8> = Vec::new();
                loop {
                    let blk = {
                        lock_ops += 1;
                        let mut st = shared.state.lock().unwrap();
                        let spent = std::mem::take(&mut cur);
                        if spent.capacity() != 0 && st.free_out.len() < shared.slots as usize {
                            st.free_out.push(spent);
                        }
                        loop {
                            let taken = st.taken;
                            if let Some(res) = st.ready.remove(&taken) {
                                st.taken += 1; shared.cv.notify_all(); break Some(res);
                            }
                            if st.end == Some(st.taken) { break None; }
                            st = shared.cv.wait(st).unwrap();
                        }
                    };
                    match blk {
                        None => break,
                        Some(Ok(v)) => {
                            on_bytes(&v);
                            total += v.len();
                            cur = v;
                        }
                        Some(Err(e)) => {
                            shared.state.lock().unwrap().cancelled = true;
                            shared.cv.notify_all();
                            let _ = feeder.join(); let _ = scanner_h.join();
                            for h in whs { let _ = h.join(); }
                            return Err(e);
                        }
                    }
                }
            }
        }
        let _ = feeder.join();
        let _ = scanner_h.join();
        for h in whs { let _ = h.join(); }
        Ok((total, lock_ops))
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

// Exact ab.rs 4-lane Hash (32 bytes/iter) so reader cost matches real i9 `ab` runs!
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
            self.nt += k;
            b = &b[k..];
            if self.nt < 32 { return; }
            let t = self.tail;
            self.block(&t);
            self.nt = 0;
        }
        let mut c = b.chunks_exact(32);
        for blk in &mut c {
            let arr: &[u8; 32] = blk.try_into().unwrap();
            self.block(arr);
        }
        let rem = c.remainder();
        self.tail[..rem.len()].copy_from_slice(rem);
        self.nt = rem.len();
    }
    fn finish(&mut self) -> u64 {
        if self.nt > 0 {
            self.tail[self.nt..].fill(0);
            let t = self.tail;
            self.block(&t);
            self.nt = 0;
        }
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

fn bench_rle_crc_micro() {
    // Extract the 6 authenticated blocks' raw BWT-inverted (pre-RLE1) bytes + stored CRC
    let path = "/tmp/wormhole_r5/wikidata_60b.bz2";
    let data = fs::read(path).unwrap();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut scratch = Scratch::<OpenCells>::default();
    let mut raw_rle_blocks: Vec<(Vec<u8>, u32, usize)> = Vec::new();
    // Re-encode the decoded blocks into RLE1 (exact bzip2 RLE1) or invert block-*.L directly!
    let mut out = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
        out.clear();
        let crc = mibz2::decode_block(&symbols, k, &mut scratch, &mut out).unwrap();
        // RLE1-encode `out` back to exact pre-RLE1 bytes
        let mut rle = Vec::with_capacity(out.len());
        let mut i = 0;
        let mut runs4 = 0usize;
        while i < out.len() {
            let c = out[i];
            let mut j = i + 1;
            while j < out.len() && out[j] == c && (j - i) < 259 { j += 1; }
            let len = j - i;
            if len >= 4 {
                runs4 += 1;
                rle.extend_from_slice(&[c, c, c, c, (len - 4) as u8]);
            } else {
                for _ in 0..len { rle.push(c); }
            }
            i = j;
        }
        raw_rle_blocks.push((rle, crc, runs4));
        syms = symbols.syms;
        if raw_rle_blocks.len() == 6 { break; }
    }
    println!("=== PART 1: RLE1 + CRC32 STAGE MICROBENCH (6 auth Wikidata blocks x 39 = 234 blocks, 210.6 MB) ===");
    for (idx, (rle, crc, runs4)) in raw_rle_blocks.iter().enumerate() {
        let mut o1 = Vec::new();
        safe_c33::unrle_span_safe(rle, &mut o1).unwrap();
        let c_sb8 = !safe_c33::crc32_sb8(!0, &o1);
        let c_sb16 = !safe_c33::crc32_sb16(!0, &o1);
        let mut o2 = Vec::new();
        let c_fused = safe_c33::unrle_and_crc_tiled(rle, &mut o2, 32 << 10, true).unwrap();
        assert_eq!(o1, o2);
        assert_eq!(c_sb8, *crc);
        assert_eq!(c_sb16, *crc);
        assert_eq!(c_fused, *crc);
        println!("  blk {idx}: rle={} -> out={} bytes | runs4={runs4} | CRC={crc:08x} EXACT", rle.len(), o1.len());
    }

    // Time 15 rotated rounds over 234 blocks (39x6):
    // M_sep_sb8 (separate unrle_span + sb8), M_sep_sb16, C2_tile32k_sb8, C2_tile32k_sb16, C2_tile64k_sb16
    let names = ["Sep_Span+SB8", "Sep_Span+SB16", "Fused_32K+SB8", "Fused_32K+SB16", "Fused_16K+SB16"];
    let mut times: Vec<Vec<f64>> = vec![Vec::new(); names.len()];
    let mut out_buf = Vec::with_capacity(960_000);
    for r in 0..15 {
        for off in 0..names.len() {
            let m = (r + off) % names.len();
            let t0 = Instant::now();
            let mut acc = 0u32;
            for _ in 0..39 {
                for (rle, crc, _) in &raw_rle_blocks {
                    out_buf.clear();
                    let got = match m {
                        0 => { safe_c33::unrle_span_safe(rle, &mut out_buf).unwrap(); !safe_c33::crc32_sb8(!0, &out_buf) }
                        1 => { safe_c33::unrle_span_safe(rle, &mut out_buf).unwrap(); !safe_c33::crc32_sb16(!0, &out_buf) }
                        2 => safe_c33::unrle_and_crc_tiled(rle, &mut out_buf, 32 << 10, false).unwrap(),
                        3 => safe_c33::unrle_and_crc_tiled(rle, &mut out_buf, 32 << 10, true).unwrap(),
                        4 => safe_c33::unrle_and_crc_tiled(rle, &mut out_buf, 16 << 10, true).unwrap(),
                        _ => unreachable!(),
                    };
                    debug_assert_eq!(got, *crc);
                    acc ^= got;
                }
            }
            std::hint::black_box(acc);
            times[m].push(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    let base = &times[0];
    for (m, name) in names.iter().enumerate() {
        let mut v = times[m].clone(); v.sort_by(|a,b| a.total_cmp(b));
        let pairs: Vec<(f64, f64)> = base.iter().copied().zip(times[m].iter().copied()).collect();
        let (rat, lo, hi) = ci95_ratio(&pairs);
        println!("  {:<18} med={:>7.2} ms  ratio vs Sep_Span+SB8: {:.4} [{:.4}, {:.4}]", name, v[7], rat, lo, hi);
    }
}

fn run_e2e_arm(arm: &str, data: Vec<u8>, w: usize) -> (f64, f64, f64, i64, f64, usize, u64, usize) {
    let k = Kernels::detect();
    let r0 = rusage();
    let t0 = Instant::now();
    let mut h = AbHash::new();
    let mut bytes = 0usize;
    let mut locks = 0usize;
    match arm {
        "MasterA" | "MasterB" => {
            let mut rdr = BzReader::parallel_workers(Cursor::new(data), w, 4 * w, k, true);
            let mut buf = vec![0u8; 4 << 20];
            loop {
                let n = rdr.read(&mut buf).unwrap();
                if n == 0 { break; }
                h.update(&buf[..n]);
                bytes += n;
                locks += 1;
            }
            drop(rdr);
        }
        "C32_OneBlkCopy" => {
            let (b, l) = safe_c33::decode_c33_e2e(Cursor::new(data), w, safe_c33::ReadMode::OneBlockCopy, k, |c| h.update(c)).unwrap();
            bytes = b; locks = l;
        }
        "C33a_BatchFill" => {
            let (b, l) = safe_c33::decode_c33_e2e(Cursor::new(data), w, safe_c33::ReadMode::BatchFillCopy, k, |c| h.update(c)).unwrap();
            bytes = b; locks = l;
        }
        "C33b_ZeroCopy" => {
            let (b, l) = safe_c33::decode_c33_e2e(Cursor::new(data), w, safe_c33::ReadMode::ZeroCopyBufRead, k, |c| h.update(c)).unwrap();
            bytes = b; locks = l;
        }
        _ => panic!("unknown arm {arm}"),
    }
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    let r1 = rusage();
    (wall, tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
     r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss as f64 / 1024.0, bytes, h.finish(), locks)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let path = &args[2];
        let arm = &args[3];
        let w: usize = args[4].parse().unwrap();
        let data = fs::read(path).unwrap();
        let (wall, u, s, flt, rss, bytes, h, locks) = run_e2e_arm(arm, data, w);
        println!("{wall:.3} {u:.3} {s:.3} {flt} {rss:.2} {bytes} {h:016x} {locks}");
        return;
    }
    bench_rle_crc_micro();

    let exe = std::env::current_exe().unwrap();
    let arms = ["MasterA", "MasterB", "C32_OneBlkCopy", "C33a_BatchFill", "C33b_ZeroCopy"];
    let path210 = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";
    println!("\n=== PART 2: 12-ROUND ROTATED FRESH-PROCESS 4W A/B (wikidata_431b_210m.bz2, 211 MB, AbHash) ===");
    let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut users: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut lcks: BTreeMap<&str, usize> = BTreeMap::new();
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
            rsss.entry(arm).or_default().push(p[4].parse().unwrap());
            lcks.insert(arm, p[7].parse().unwrap());
        }
    }
    let ma = &walls["MasterA"];
    println!("{:<16} {:>8} {:>24} {:>8} {:>8} {:>8} {:>8} {:>6}", "Arm", "med_ms", "ratio vs MasterA [95%]", "user_ms", "sys_ms", "minflt", "rss_MiB", "locks");
    for &arm in &arms {
        let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
        let mut uv = users[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
        let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
        let mut fv = flts[arm].clone(); fv.sort();
        let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
        let pairs: Vec<(f64, f64)> = ma.iter().copied().zip(walls[arm].iter().copied()).collect();
        let (rat, lo, hi) = ci95_ratio(&pairs);
        println!("{:<16} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>8.2} {:>8.2} {:>8} {:>8.2} {:>6}",
            arm, wv[6], rat, lo, hi, uv[6], sv[6], fv[6], rv[6], lcks[arm]);
    }

    println!("\n=== PART 3: COLD-START W=10 OS & WALL CHECK (wikidata_431b_210m.bz2, 3-rd median) ===");
    println!("{:<16} {:>8} {:>8} {:>8} {:>8} {:>8} {:>6}", "Arm", "wall_ms", "user_ms", "sys_ms", "minflt", "rss_MiB", "locks");
    for &arm in &arms {
        let mut rows = Vec::new();
        for _ in 0..3 {
            let out = Command::new(&exe).args(["--child", path210, arm, "10"]).output().unwrap();
            let s = String::from_utf8(out.stdout).unwrap();
            let p: Vec<&str> = s.split_whitespace().collect();
            assert_eq!(p[6], ref_hash);
            rows.push((p[0].parse::<f64>().unwrap(), p[1].parse::<f64>().unwrap(), p[2].parse::<f64>().unwrap(),
                       p[3].parse::<i64>().unwrap(), p[4].parse::<f64>().unwrap(), p[7].parse::<usize>().unwrap()));
        }
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
        let m = rows[1];
        println!("{:<16} {:>8.2} {:>8.2} {:>8.2} {:>8} {:>8.2} {:>6}", arm, m.0, m.1, m.2, m.3, m.4, m.5);
    }
}
