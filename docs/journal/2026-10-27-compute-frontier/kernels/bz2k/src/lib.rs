//! A standalone bzip2 decoder, written to time each phase of the decode apart and to try
//! the inverse BWT interleaved across blocks. Not production code: no randomised blocks,
//! errors are panics. Each phase is its own pass over the block so it can be timed:
//!
//! 1. `huffman`: the block's Huffman codes to MTF/RLE2 symbols (u16), selectors applied;
//! 2. `mtf`: RUNA/RUNB runs and move-to-front, to the BWT's last column (u8), and counts;
//! 3. `build`: the counting sort's cumulative counts and the T-vector (the inverse BWT's
//!    linked list, packed as lbzip2 packs it: the byte in bits 0-7, the next index above);
//! 4. `walk`: the list followed from the primary index, the RLE1-coded bytes out;
//! 5. `unrle_crc`: RLE1 undone and the block CRC (CRC-32/BZIP2) taken.
//!
//! `decode_fused` does 4 and 5 in one loop as lbzip2's `emit` does.

pub mod crc;
pub mod simd;

pub struct BitReader<'a> {
    data: &'a [u8],
    pos: usize, // next byte to load
    buf: u64,   // bits, MSB first, `live` of them valid at the top
    live: u32,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BitReader { data, pos: 0, buf: 0, live: 0 }
    }
    #[inline(always)]
    fn refill(&mut self) {
        while self.live <= 56 {
            let b = if self.pos < self.data.len() { self.data[self.pos] } else { 0 };
            self.pos += 1;
            self.buf |= (b as u64) << (56 - self.live);
            self.live += 8;
        }
    }
    #[inline(always)]
    pub fn peek(&mut self, n: u32) -> u32 {
        if self.live < n {
            self.refill();
        }
        (self.buf >> (64 - n)) as u32
    }
    #[inline(always)]
    pub fn skip(&mut self, n: u32) {
        self.buf <<= n;
        self.live -= n;
    }
    #[inline(always)]
    pub fn bits(&mut self, n: u32) -> u32 {
        let v = self.peek(n);
        self.skip(n);
        v
    }
    pub fn bit(&mut self) -> bool {
        self.bits(1) == 1
    }
    /// Bits consumed so far
    pub fn bit_pos(&self) -> u64 {
        self.pos as u64 * 8 - self.live as u64
    }
    pub fn exhausted(&self) -> bool {
        self.bit_pos() >= self.data.len() as u64 * 8
    }
    pub fn align_byte(&mut self) {
        let r = self.live % 8;
        self.skip(r);
    }
}

const LUT_BITS: u32 = 10;

/// A group's canonical Huffman code: a direct table for codes up to LUT_BITS bits, the
/// rest by canonical limits (as bzip2's limit/base/perm)
pub struct Huff {
    lut: Vec<u32>, // (len << 16) | sym, len 0 = longer than LUT_BITS
    limit: [i32; 22],
    base: [i32; 22],
    perm: Vec<u16>,
    min_len: u32,
    max_len: u32,
}

impl Huff {
    fn new(lens: &[u8]) -> Huff {
        let min_len = *lens.iter().min().unwrap() as u32;
        let max_len = *lens.iter().max().unwrap() as u32;
        let mut perm = Vec::with_capacity(lens.len());
        for l in min_len..=max_len {
            for (s, &x) in lens.iter().enumerate() {
                if x as u32 == l {
                    perm.push(s as u16);
                }
            }
        }
        let mut count = [0i32; 22];
        for &x in lens {
            count[x as usize] += 1;
        }
        let mut limit = [-1i32; 22];
        let mut base = [0i32; 22];
        let mut code = 0i32;
        let mut idx = 0i32; // index into perm of the first code of this length
        let mut lut = vec![0u32; 1 << LUT_BITS];
        for l in min_len..=max_len {
            let n = count[l as usize];
            base[l as usize] = idx - code; // perm index = code + base
            // the codes of length l: code..code+n
            if l <= LUT_BITS {
                for k in 0..n {
                    let c = (code + k) as u32;
                    let sym = perm[(idx + k) as usize] as u32;
                    let lo = c << (LUT_BITS - l);
                    let hi = (c + 1) << (LUT_BITS - l);
                    for e in lo..hi {
                        lut[e as usize] = (l << 16) | sym;
                    }
                }
            }
            code += n;
            idx += n;
            limit[l as usize] = code - 1;
            code <<= 1;
        }
        Huff { lut, limit, base, perm, min_len, max_len }
    }

    #[inline(always)]
    fn decode(&self, br: &mut BitReader) -> u16 {
        let w = br.peek(LUT_BITS);
        let e = self.lut[w as usize];
        let l = e >> 16;
        if l != 0 {
            br.skip(l);
            return e as u16;
        }
        // longer than the table: canonical, bit length by bit length
        let mut l = LUT_BITS + 1;
        loop {
            let c = (br.peek(l)) as i32;
            if c <= self.limit[l as usize] {
                br.skip(l);
                return self.perm[(c + self.base[l as usize]) as usize];
            }
            l += 1;
            assert!(l <= self.max_len, "bad Huffman code");
        }
    }
}

/// A block's header and its symbols (the output of phase 1), or the stream's end
pub struct Block {
    pub crc: u32,
    pub orig_ptr: u32,
    pub seq_to_unseq: Vec<u8>,
    pub syms: Vec<u16>,
    pub eob: u16,
}

pub enum Next {
    Block(Block),
    End(u32),
}

/// Read the next block of the stream (phase 1, Huffman decoding included), or its end.
/// `syms` is reused.
pub fn read_block(br: &mut BitReader, mut syms: Vec<u16>) -> Next {
    let magic = (br.bits(24) as u64) << 24 | br.bits(24) as u64;
    if magic == 0x177245385090 {
        let crc = br.bits(32);
        return Next::End(crc);
    }
    assert_eq!(magic, 0x314159265359, "bad block magic");
    let crc = br.bits(32);
    assert!(!br.bit(), "randomised blocks are not supported");
    let orig_ptr = br.bits(24);
    let in_use16 = br.bits(16);
    let mut seq_to_unseq = Vec::with_capacity(256);
    for i in 0..16 {
        if in_use16 & (0x8000 >> i) != 0 {
            let w = br.bits(16);
            for j in 0..16 {
                if w & (0x8000 >> j) != 0 {
                    seq_to_unseq.push((i * 16 + j) as u8);
                }
            }
        }
    }
    let alpha = seq_to_unseq.len() + 2;
    let n_groups = br.bits(3) as usize;
    assert!((2..=6).contains(&n_groups));
    let n_sel = br.bits(15) as usize;
    let mut mtf_g: Vec<u8> = (0..n_groups as u8).collect();
    let mut selectors = Vec::with_capacity(n_sel);
    for _ in 0..n_sel {
        let mut j = 0;
        while br.bit() {
            j += 1;
            assert!(j < n_groups);
        }
        let v = mtf_g[j];
        mtf_g.copy_within(0..j, 1);
        mtf_g[0] = v;
        selectors.push(v);
    }
    let mut tables = Vec::with_capacity(n_groups);
    for _ in 0..n_groups {
        let mut curr = br.bits(5) as i32;
        let mut lens = vec![0u8; alpha];
        for l in lens.iter_mut() {
            loop {
                assert!((1..=20).contains(&curr));
                if !br.bit() {
                    break;
                }
                if br.bit() {
                    curr -= 1
                } else {
                    curr += 1
                }
            }
            *l = curr as u8;
        }
        tables.push(Huff::new(&lens));
    }
    let eob = (alpha - 1) as u16;
    syms.clear();
    let mut sel = 0;
    'outer: loop {
        let t = &tables[selectors[sel] as usize];
        sel += 1;
        for _ in 0..50 {
            let s = t.decode(br);
            syms.push(s);
            if s == eob {
                break 'outer;
            }
        }
    }
    Next::Block(Block { crc, orig_ptr, seq_to_unseq, syms, eob })
}

/// Phase 2: RUNA/RUNB runs and move-to-front, into `ll` (the BWT's last column) and the
/// byte counts. Returns the block size.
pub fn mtf(b: &Block, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> usize {
    ll.clear();
    *counts = [0; 256];
    let mut list = [0u8; 256];
    for (i, x) in list.iter_mut().enumerate() {
        *x = i as u8;
    }
    let mut run: u32 = 0;
    let mut run_bit: u32 = 0;
    for &s in &b.syms {
        if s <= 1 {
            run += (s as u32 + 1) << run_bit;
            run_bit += 1;
            continue;
        }
        if run_bit != 0 {
            let c = b.seq_to_unseq[list[0] as usize];
            counts[c as usize] += run;
            ll.resize(ll.len() + run as usize, c);
            run = 0;
            run_bit = 0;
        }
        if s == b.eob {
            break;
        }
        let nn = (s - 1) as usize;
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let c = b.seq_to_unseq[v as usize];
        counts[c as usize] += 1;
        ll.push(c);
    }
    assert!(ll.len() <= 900_000);
    ll.len()
}

/// Phase 3: the T-vector (lbzip2's `decode`): tt[i] = (next << 8) | byte
pub fn build(ll: &[u8], counts: &[u32; 256], tt: &mut Vec<u32>) {
    tt.clear();
    tt.extend(ll.iter().map(|&c| c as u32));
    let mut ftab = [0u32; 256];
    let mut cum = 0;
    for i in 0..256 {
        ftab[i] = cum;
        cum += counts[i];
    }
    for i in 0..ll.len() {
        let c = ll[i] as usize;
        let k = ftab[c] as usize;
        tt[k] += (i as u32) << 8;
        ftab[c] += 1;
    }
}

/// Phase 4: follow the list from the primary index: the RLE1-coded block
#[inline(never)]
pub fn walk(tt: &[u32], orig_ptr: u32, out: &mut Vec<u8>) {
    out.clear();
    out.reserve(tt.len());
    let mut p = tt[orig_ptr as usize];
    for _ in 0..tt.len() {
        p = tt[(p >> 8) as usize];
        out.push(p as u8);
    }
}

/// Phase 4 for K blocks at once: their lists followed in lockstep, so K independent loads
/// are in flight. Each out[k] gets its block's RLE1-coded bytes.
pub fn walk_k<const K: usize>(tts: [&[u32]; K], ptrs: [u32; K], outs: &mut [Vec<u8>; K]) {
    let mut p = [0u32; K];
    let mut n = [0usize; K];
    for k in 0..K {
        outs[k].clear();
        outs[k].resize(tts[k].len(), 0);
        p[k] = tts[k][ptrs[k] as usize];
        n[k] = tts[k].len();
    }
    let common = *n.iter().min().unwrap();
    for i in 0..common {
        for k in 0..K {
            // (bounds checked: the list is untrusted)
            p[k] = tts[k][(p[k] >> 8) as usize];
            outs[k][i] = p[k] as u8;
        }
    }
    for k in 0..K {
        for i in common..n[k] {
            p[k] = tts[k][(p[k] >> 8) as usize];
            outs[k][i] = p[k] as u8;
        }
    }
}

/// Phase 5: RLE1 undone (4 equal bytes then a count of more), appended to `out`; the
/// block's CRC
pub fn unrle_crc(rle: &[u8], out: &mut Vec<u8>, crc_kind: crc::Kind) -> u32 {
    let start = out.len();
    if crc_kind == crc::Kind::Simd {
        simd::unrle(rle, out);
        return simd::crc32_bzip2(&out[start..]);
    }
    let mut i = 0;
    let n = rle.len();
    out.reserve(n + n / 4);
    while i < n {
        // copy a stretch with no 4 equal bytes in a row
        let c = rle[i];
        if i + 3 < n && rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
            out.extend_from_slice(&[c, c, c, c]);
            let extra = if i + 4 < n { rle[i + 4] as usize } else { 0 };
            out.resize(out.len() + extra, c);
            i += 5;
        } else {
            out.push(c);
            i += 1;
        }
    }
    crc::crc32_bzip2(&out[start..], crc_kind)
}

/// Phases 4 and 5 fused, as lbzip2's `emit`: the list followed, RLE1 undone and the CRC
/// taken a byte at a time by table
pub fn walk_unrle_crc_fused(tt: &[u32], orig_ptr: u32, out: &mut Vec<u8>) -> u32 {
    let t = &crc::TABLE;
    let mut s: u32 = !0;
    let mut p = tt[orig_ptr as usize];
    let mut prev: i32 = -1;
    let mut same = 0;
    let mut left = tt.len();
    while left > 0 {
        p = tt[(p >> 8) as usize];
        left -= 1;
        let c = p as u8;
        if same == 4 {
            for _ in 0..c {
                out.push(prev as u8);
                s = (s << 8) ^ t[((s >> 24) as u8 ^ prev as u8) as usize];
            }
            same = 0;
            prev = -1;
            continue;
        }
        out.push(c);
        s = (s << 8) ^ t[((s >> 24) as u8 ^ c) as usize];
        if c as i32 == prev {
            same += 1;
        } else {
            same = 1;
            prev = c as i32;
        }
    }
    !s
}

#[inline(always)]
pub fn rdtsc() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        std::time::Instant::now().elapsed().as_nanos() as u64
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Cycles {
    pub huffman: u64,
    pub mtf: u64,
    pub build: u64,
    pub walk: u64,
    pub unrle_crc: u64,
    pub blocks: u64,
    pub in_bits: u64,
    pub rle_bytes: u64,
    pub out_bytes: u64,
}

impl Cycles {
    pub fn add(&mut self, o: &Cycles) {
        self.huffman += o.huffman;
        self.mtf += o.mtf;
        self.build += o.build;
        self.walk += o.walk;
        self.unrle_crc += o.unrle_crc;
        self.blocks += o.blocks;
        self.in_bits += o.in_bits;
        self.rle_bytes += o.rle_bytes;
        self.out_bytes += o.out_bytes;
    }
}

/// The stream header; the reader is then at the first block
pub fn stream_header(br: &mut BitReader) -> bool {
    if br.exhausted() {
        return false;
    }
    let h = br.bits(24);
    if h != 0x425a68 {
        return false;
    }
    let lvl = br.bits(8);
    assert!((b'1' as u32..=b'9' as u32).contains(&lvl));
    true
}

/// The blocks' phase-2 output (each block's `ll`, counts, primary index and CRC), for
/// kernels that start from it
pub struct Prepared {
    pub ll: Vec<u8>,
    pub counts: [u32; 256],
    pub orig_ptr: u32,
    pub crc: u32,
}

/// Decode a whole .bz2 (concatenated streams too), phase by phase, timing each;
/// `out` gets the decoded bytes (if `keep`), and `prepared` each block's phase-2 output
/// (if given). Every block's CRC and every stream's combined CRC are checked.
pub fn decode_all(data: &[u8], out: &mut Vec<u8>, keep: bool, mut prepared: Option<&mut Vec<Prepared>>,
                  crc_kind: crc::Kind) -> Cycles {
    let mut br = BitReader::new(data);
    let mut cy = Cycles::default();
    let mut syms = Vec::new();
    let mut ll = Vec::new();
    let mut tt = Vec::new();
    let mut rle = Vec::new();
    let mut counts = [0u32; 256];
    while stream_header(&mut br) {
        let mut combined: u32 = 0;
        loop {
            let b0 = br.bit_pos();
            let t0 = rdtsc();
            let next = read_block(&mut br, syms);
            let t1 = rdtsc();
            cy.in_bits += br.bit_pos() - b0;
            let b = match next {
                Next::End(crc) => {
                    assert_eq!(crc, combined, "stream CRC");
                    syms = Vec::new();
                    break;
                }
                Next::Block(b) => b,
            };
            let n = if crc_kind == crc::Kind::Simd { simd::mtf(&b, &mut ll, &mut counts) } else { mtf(&b, &mut ll, &mut counts) };
            let t2 = rdtsc();
            build(&ll, &counts, &mut tt);
            let t3 = rdtsc();
            do_walk(&tt, b.orig_ptr, &mut rle);
            let t4 = rdtsc();
            if !keep {
                out.clear();
            }
            let before = out.len();
            let crc = unrle_crc(&rle, out, crc_kind);
            let t5 = rdtsc();
            assert_eq!(crc, b.crc, "block CRC");
            combined = combined.rotate_left(1) ^ crc;
            cy.huffman += t1 - t0;
            cy.mtf += t2 - t1;
            cy.build += t3 - t2;
            cy.walk += t4 - t3;
            cy.unrle_crc += t5 - t4;
            cy.blocks += 1;
            cy.rle_bytes += n as u64;
            cy.out_bytes += (out.len() - before) as u64;
            if let Some(p) = prepared.as_deref_mut() {
                p.push(Prepared { ll: ll.clone(), counts, orig_ptr: b.orig_ptr, crc: b.crc });
            }
            syms = b.syms;
        }
        br.align_byte();
    }
    cy
}

/// `walk_k` without bounds checks, the lists' and outputs' base pointers held in locals
/// (each list index is checked against its length up front: a list's links are below
/// its length by construction in `build`)
#[inline(never)]
pub fn walk_k_raw<const K: usize>(tts: [&[u32]; K], ptrs: [u32; K], outs: &mut [Vec<u8>; K]) {
    let mut p = [0u32; K];
    let mut n = [0usize; K];
    let mut t: [*const u32; K] = [std::ptr::null(); K];
    let mut o: [*mut u8; K] = [std::ptr::null_mut(); K];
    for k in 0..K {
        outs[k].clear();
        outs[k].resize(tts[k].len(), 0);
        p[k] = tts[k][ptrs[k] as usize];
        n[k] = tts[k].len();
        t[k] = tts[k].as_ptr();
        o[k] = outs[k].as_mut_ptr();
    }
    let common = *n.iter().min().unwrap();
    unsafe {
        for i in 0..common {
            for k in 0..K {
                p[k] = *t[k].add((p[k] >> 8) as usize);
                *o[k].add(i) = p[k] as u8;
            }
        }
        for k in 0..K {
            for i in common..n[k] {
                p[k] = *t[k].add((p[k] >> 8) as usize);
                *o[k].add(i) = p[k] as u8;
            }
        }
    }
}

/// Every link of a T-vector is below its length (so `walk_k_raw` is in bounds)
pub fn links_in_bounds(tt: &[u32]) -> bool {
    let n = tt.len() as u32;
    tt.iter().all(|&x| (x >> 8) < n)
}

extern "C" {
    fn madvise(addr: *mut core::ffi::c_void, len: usize, advice: i32) -> i32;
}

/// A copy of `v` in memory advised for transparent huge pages (2 MiB aligned)
pub fn huge_copy(v: &[u32]) -> &'static mut [u32] {
    let bytes = (v.len() * 4).div_ceil(2 << 20) * (2 << 20);
    let layout = std::alloc::Layout::from_size_align(bytes, 2 << 20).unwrap();
    unsafe {
        let p = std::alloc::alloc(layout);
        madvise(p as *mut _, bytes, 14); // MADV_HUGEPAGE
        let s = std::slice::from_raw_parts_mut(p as *mut u32, v.len());
        s.copy_from_slice(v);
        s
    }
}

/// K chains through one block's list, from K rows spread over it, n/K steps each: the
/// memory-level parallelism a split walk of one block could have (not a decode: the
/// segments are not joined)
#[inline(never)]
pub fn walk_multi<const K: usize>(tt: &[u32], out: &mut [u8]) {
    let n = tt.len();
    let steps = n / K;
    let t = tt.as_ptr();
    let mut p: [u32; K] = std::array::from_fn(|k| tt[k * steps]);
    let o = out.as_mut_ptr();
    unsafe {
        for i in 0..steps {
            for k in 0..K {
                p[k] = *t.add((p[k] >> 8) as usize);
                *o.add(k * steps + i) = p[k] as u8;
            }
        }
    }
}

/// The inverse BWT's walk split into segments walked K at a time (memory-level
/// parallelism within one block), then joined: exactly `walk`'s output.
///
/// The list is a cycle through every row (unless the block is a string repeated, when it
/// is several: then None, and the caller walks it whole). M rows on it are chosen as
/// segment starts, the walk's own first row among them; each segment is the walk from its
/// start until the next row is a start. K lanes walk segments in lockstep, each taking the
/// next unwalked segment when its own ends. The segments are then joined in the order the
/// starts follow one another from the first.
#[inline(never)]
pub fn walk_segments<const K: usize>(tt: &[u32], orig_ptr: u32, m: usize, out: &mut Vec<u8>) -> bool {
    let n = tt.len();
    out.clear();
    if n == 0 {
        return true;
    }
    let r0 = (tt[orig_ptr as usize] >> 8) as usize;
    // the starts: r0 and m-1 rows spread evenly over the rows
    let mut starts: Vec<u32> = random_rows(n, m - 1).into_iter().filter(|&r| r as usize != r0).collect();
    starts.push(r0 as u32);
    starts.sort_unstable();
    starts.dedup();
    let ns = starts.len();
    let mut bitmap = vec![0u64; n.div_ceil(64)];
    for &s in &starts {
        bitmap[(s >> 6) as usize] |= 1 << (s & 63);
    }
    let seg_of = |row: u32| starts.binary_search(&row).unwrap();
    let cap = 2 * n / ns + 64;
    let mut bufs: Vec<Vec<u8>> = (0..ns).map(|_| Vec::with_capacity(cap)).collect();
    let mut next = vec![usize::MAX; ns];
    let mut lane_seg = [usize::MAX; K];
    let mut lane_row = [0u32; K];
    let mut queued = 0;
    for k in 0..K {
        if queued < ns {
            lane_seg[k] = queued;
            lane_row[k] = starts[queued];
            queued += 1;
        }
    }
    let mut active = (0..K).filter(|&k| lane_seg[k] != usize::MAX).count();
    let mut steps = 0usize;
    let t = tt.as_ptr();
    let bm = bitmap.as_ptr();
    while active > 0 {
        for k in 0..K {
            let seg = lane_seg[k];
            if seg == usize::MAX {
                continue;
            }
            // (rows are below n: checked by links_in_bounds before)
            let q = unsafe { *t.add(lane_row[k] as usize) };
            let row = q >> 8;
            unsafe { bufs.get_unchecked_mut(seg) }.push(q as u8);
            lane_row[k] = row;
            if unsafe { *bm.add((row >> 6) as usize) } >> (row & 63) & 1 != 0 {
                next[seg] = seg_of(row);
                if queued < ns {
                    lane_seg[k] = queued;
                    lane_row[k] = starts[queued];
                    queued += 1;
                } else {
                    lane_seg[k] = usize::MAX;
                    active -= 1;
                }
            }
        }
        steps += K;
        if steps > 4 * n + 4 * K {
            return false; // a repeated block: several cycles
        }
    }
    // join from r0's segment; every segment once, n bytes in all
    let first = seg_of(r0 as u32);
    let total: usize = bufs.iter().map(|b| b.len()).sum();
    if total != n {
        return false;
    }
    out.reserve(n);
    let mut s = first;
    for _ in 0..ns {
        out.extend_from_slice(&bufs[s]);
        s = next[s];
    }
    s == first && out.len() == n
}

/// The walk `BZ2K_WALK` names: `seg` (walk_segments, 12 lanes, 256 segments, falling back
/// to `walk` for a repeated block) or the plain walk
pub fn do_walk(tt: &[u32], orig_ptr: u32, out: &mut Vec<u8>) {
    static SEG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *SEG.get_or_init(|| std::env::var("BZ2K_WALK").as_deref() == Ok("seg")) {
        assert!(links_in_bounds(tt));
        if walk_segments::<12>(tt, orig_ptr, 256, out) {
            return;
        }
    }
    walk(tt, orig_ptr, out)
}

/// One block from the reader (at its magic), phase by phase, its CRC checked
pub fn decode_one(br: &mut BitReader, out: &mut Vec<u8>, crc_kind: crc::Kind) -> Cycles {
    let mut cy = Cycles::default();
    let (mut ll, mut tt, mut rle) = (Vec::new(), Vec::new(), Vec::new());
    let mut counts = [0u32; 256];
    let b0 = br.bit_pos();
    let t0 = rdtsc();
    let b = match read_block(br, Vec::new()) {
        Next::Block(b) => b,
        Next::End(_) => return cy,
    };
    let t1 = rdtsc();
    cy.in_bits = br.bit_pos() - b0;
    let n = mtf(&b, &mut ll, &mut counts);
    let t2 = rdtsc();
    build(&ll, &counts, &mut tt);
    let t3 = rdtsc();
    do_walk(&tt, b.orig_ptr, &mut rle);
    let t4 = rdtsc();
    let crc = unrle_crc(&rle, out, crc_kind);
    let t5 = rdtsc();
    assert_eq!(crc, b.crc, "block CRC");
    cy.huffman = t1 - t0;
    cy.mtf = t2 - t1;
    cy.build = t3 - t2;
    cy.walk = t4 - t3;
    cy.unrle_crc = t5 - t4;
    cy.blocks = 1;
    cy.rle_bytes = n as u64;
    cy.out_bytes = out.len() as u64;
    cy
}

/// `m` pseudo-random rows below `n` (xorshift): a uniformly random row is a uniformly
/// random position of the block's text, so the segments' lengths are about equal in
/// expectation, where evenly spaced rows (in sorted order) can cluster in the text
fn random_rows(n: usize, m: usize) -> Vec<u32> {
    let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
    (0..m)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x % n as u64) as u32
        })
        .collect()
}
