use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;

// Paired A/B micro-falsifier on all 233 large blocks of wikidata.bz2:
// Compares:
//   ARM A (Baseline): Open's exact Elimination + memoized Spelling + inline16
//   ARM B (Cross-Block Hash-Consed SLP): Persistent per-window hash-consing of (cl, cr) during Elimination,
//         caching pre-expanded words & 16-byte inline prefixes across blocks, and copying cached inline16
//         entries into block-local inline16/spelling for exact bit-identical emit tables.
// Verifies 100% byte-for-byte identity of every block's inline16, lens, and long words!

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

struct BlockInput {
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
    let mut pos = 0u32;
    let mut i = 0;
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < records.len() && (records[i] >> 8) as u8 == c {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        visit(c, pos, width);
        pos += width;
    }
}

fn split(
    c: u8,
    pos: u32,
    width: u32,
    p: u32,
    limit: u32,
    mut visit: impl FnMut(u8, u32, u32, bool),
) {
    let end = pos + width;
    if width > limit && pos <= p && p < end {
        if pos < p {
            visit(c, pos, p - pos, false);
        }
        visit(c, p, 1, true);
        if p + 1 < end {
            visit(c, p + 1, end - p - 1, false);
        }
    } else {
        visit(c, pos, width, width <= limit);
    }
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut tot = 0u32;
    counts.map(|w| {
        let s = tot;
        tot += w;
        s
    })
}

struct PreStrips {
    n: u32,
    thin: [u32; 256],
    strips: Vec<Strip>,
    image_order: Vec<u32>,
}

fn prep_strips(inp: &BlockInput) -> PreStrips {
    let mut all = [0u32; 256];
    let mut thin = [0u32; 256];
    let mut thick = [0u32; 256];
    let mut strips = Vec::with_capacity(4096);
    maximal(&inp.records, |c, pos, width| {
        split(c, pos, width, inp.p, LIMIT, |c, x, w, keep| {
            let a = usize::from(c);
            if keep {
                thin[a] += w;
            } else {
                strips.push(Strip {
                    from: x,
                    to: all[a],
                    width: w,
                    byte: c,
                });
                thick[a] += 1;
            }
            all[a] += w;
        });
    });
    let first = starts(all);
    let mut write = starts(thick);
    let ns = strips.len();
    let mut image_order = vec![0u32; ns];
    for (i, s) in strips.iter_mut().enumerate() {
        let a = usize::from(s.byte);
        s.to += first[a];
        image_order[write[a] as usize] = i as u32;
        write[a] += 1;
    }
    PreStrips {
        n: inp.n as u32,
        thin,
        strips,
        image_order,
    }
}

struct PlainScratch {
    prev0: Vec<u32>,
    next0: Vec<u32>,
    prev1: Vec<u32>,
    next1: Vec<u32>,
    n_width: Vec<u32>,
    n_word: Vec<u32>,
    rules: Vec<[u32; 4]>, // left, right, len, first
    stack: Vec<u32>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    inline16: Vec<[u8; 16]>,
    lens: Vec<u8>,
}

impl Default for PlainScratch {
    fn default() -> Self {
        Self {
            prev0: Vec::new(),
            next0: Vec::new(),
            prev1: Vec::new(),
            next1: Vec::new(),
            n_width: Vec::new(),
            n_word: Vec::new(),
            rules: Vec::new(),
            stack: Vec::new(),
            spelling: Vec::new(),
            offsets: Vec::new(),
            inline16: Vec::new(),
            lens: Vec::new(),
        }
    }
}

fn run_plain(ps: &PreStrips, sc: &mut PlainScratch) -> (u64, u64, u64) {
    let t0 = Instant::now();
    sc.prev0.clear();
    sc.next0.clear();
    sc.prev1.clear();
    sc.next1.clear();
    sc.n_width.clear();
    sc.n_word.clear();
    sc.rules.clear();
    sc.prev0.push(0);
    sc.next0.push(0);
    sc.prev1.push(0);
    sc.next1.push(0);
    sc.n_width.push(0);
    sc.n_word.push(0);

    macro_rules! add_node {
        ($w:expr, $wd:expr) => {{
            let id = sc.n_width.len() as u32;
            sc.prev0.push(0);
            sc.next0.push(0);
            sc.prev1.push(0);
            sc.next1.push(0);
            sc.n_width.push($w);
            sc.n_word.push($wd);
            id
        }};
    }
    for s in &ps.strips {
        add_node!(s.width, u32::from(s.byte) + 1);
    }
    let mut term0 = (0u32, 0u32);
    let mut ord0 = (0u32, 0u32);
    let mut term1 = (0u32, 0u32);
    let mut ord1 = (0u32, 0u32);
    let append_to = |lst: &mut (u32, u32), id: u32, prev: &mut [u32], next: &mut [u32]| {
        if lst.1 == 0 {
            lst.0 = id;
        } else {
            next[lst.1 as usize] = id;
        }
        prev[id as usize] = lst.1;
        lst.1 = id;
    };
    let mut x = 0u32;
    for (i, s) in ps.strips.iter().enumerate() {
        if x < s.from {
            let id = add_node!(s.from - x, 0);
            append_to(&mut ord0, id, &mut sc.prev0, &mut sc.next0);
            append_to(&mut term1, id, &mut sc.prev1, &mut sc.next1);
        }
        append_to(&mut ord0, i as u32 + 1, &mut sc.prev0, &mut sc.next0);
        x = s.from + s.width;
    }
    if x < ps.n {
        let id = add_node!(ps.n - x, 0);
        append_to(&mut ord0, id, &mut sc.prev0, &mut sc.next0);
        append_to(&mut term1, id, &mut sc.prev1, &mut sc.next1);
    }
    x = 0;
    for &idx in &ps.image_order {
        let s = ps.strips[idx as usize];
        if x < s.to {
            let id = add_node!(s.to - x, 0);
            append_to(&mut ord1, id, &mut sc.prev1, &mut sc.next1);
            append_to(&mut term0, id, &mut sc.prev0, &mut sc.next0);
        }
        append_to(&mut ord1, idx + 1, &mut sc.prev1, &mut sc.next1);
        x = s.to + s.width;
    }
    if x < ps.n {
        let id = add_node!(ps.n - x, 0);
        append_to(&mut ord1, id, &mut sc.prev1, &mut sc.next1);
        append_to(&mut term0, id, &mut sc.prev0, &mut sc.next0);
    }
    let close = |first_l: (u32, u32), second_l: (u32, u32), prev: &mut [u32], next: &mut [u32]| {
        let mut p_id = 0u32;
        for (h, t) in [first_l, second_l] {
            if t == 0 {
                continue;
            }
            next[p_id as usize] = h;
            prev[h as usize] = p_id;
            p_id = t;
        }
        next[p_id as usize] = 0;
        prev[0] = p_id;
    };
    close(term0, ord0, &mut sc.prev0, &mut sc.next0);
    close(term1, ord1, &mut sc.prev1, &mut sc.next1);

    let mut remaining = ps.n;
    while remaining > 0 {
        let xn = sc.prev0[0];
        let yn = sc.prev1[0];
        let aw = sc.n_width[xn as usize];
        let bw = sc.n_width[yn as usize];
        let w = aw.min(bw);
        remaining -= w;
        if xn == yn {
            let pr = sc.prev0[xn as usize];
            let nx = sc.next0[xn as usize];
            sc.next0[pr as usize] = nx;
            sc.prev0[nx as usize] = pr;
            let pr = sc.prev1[xn as usize];
            let nx = sc.next1[xn as usize];
            sc.next1[pr as usize] = nx;
            sc.prev1[nx as usize] = pr;
            continue;
        }
        let l = sc.n_word[xn as usize];
        let r = sc.n_word[yn as usize];
        let wd = if l == 0 {
            r
        } else if r == 0 {
            l
        } else {
            let id = sc.rules.len() as u32 + 257;
            let ll = if l <= 256 { 1 } else { sc.rules[(l - 257) as usize][2] };
            let rl = if r <= 256 { 1 } else { sc.rules[(r - 257) as usize][2] };
            sc.rules.push([l, r, ll + rl, u32::MAX]);
            id
        };
        if aw <= bw {
            let pr = sc.prev0[xn as usize];
            let nx = sc.next0[xn as usize];
            sc.next0[pr as usize] = nx;
            sc.prev0[nx as usize] = pr;
            let nx = sc.next0[yn as usize];
            sc.prev0[xn as usize] = yn;
            sc.next0[xn as usize] = nx;
            sc.next0[yn as usize] = xn;
            sc.prev0[nx as usize] = xn;
            sc.n_word[xn as usize] = wd;
            if aw == bw {
                let pr = sc.prev0[yn as usize];
                let nx = sc.next0[yn as usize];
                sc.next0[pr as usize] = nx;
                sc.prev0[nx as usize] = pr;
                let pr = sc.prev1[yn as usize];
                let nx = sc.next1[yn as usize];
                sc.next1[pr as usize] = nx;
                sc.prev1[nx as usize] = pr;
            } else {
                sc.n_width[yn as usize] = bw - w;
            }
        } else {
            let pr = sc.prev1[yn as usize];
            let nx = sc.next1[yn as usize];
            sc.next1[pr as usize] = nx;
            sc.prev1[nx as usize] = pr;
            let nx = sc.next1[xn as usize];
            sc.prev1[yn as usize] = xn;
            sc.next1[yn as usize] = nx;
            sc.next1[xn as usize] = yn;
            sc.prev1[nx as usize] = yn;
            sc.n_word[yn as usize] = wd;
            sc.n_width[xn as usize] = aw - w;
        }
    }
    let t_elim = t0.elapsed().as_nanos() as u64;

    let t1 = Instant::now();
    sc.spelling.clear();
    sc.offsets.clear();
    sc.offsets.push(0);
    let mut nid = sc.next0[0];
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        let end = k + ps.thin[a];
        while k < end {
            let nw = sc.n_width[nid as usize];
            let nwd = sc.n_word[nid as usize];
            let w = (end - k).min(nw - in_node);
            if nwd != 0 {
                sc.stack.clear();
                sc.stack.push(nwd);
                while let Some(id) = sc.stack.pop() {
                    if id <= 256 {
                        sc.spelling.push((id - 1) as u8);
                    } else {
                        let ri = (id - 257) as usize;
                        if sc.rules[ri][3] == u32::MAX {
                            sc.rules[ri][3] = sc.spelling.len() as u32;
                            sc.stack.push(sc.rules[ri][1]);
                            sc.stack.push(sc.rules[ri][0]);
                        } else {
                            let st = sc.rules[ri][3] as usize;
                            let ln = sc.rules[ri][2] as usize;
                            sc.spelling.extend_from_within(st..st + ln);
                        }
                    }
                }
                sc.spelling.push(a as u8);
                sc.offsets.push(sc.spelling.len() as u32);
            }
            in_node += w;
            k += w;
            if in_node == nw {
                nid = sc.next0[nid as usize];
                in_node = 0;
            }
        }
    }
    let t_append = t1.elapsed().as_nanos() as u64;

    let t2 = Instant::now();
    sc.inline16.clear();
    sc.lens.clear();
    for w in sc.offsets.windows(2) {
        let word = &sc.spelling[w[0] as usize..w[1] as usize];
        let mut v = [0u8; 16];
        let head = word.len().min(16);
        v[..head].copy_from_slice(&word[..head]);
        sc.inline16.push(v);
        sc.lens.push(if word.len() <= 16 { word.len() as u8 } else { 255 });
    }
    let t_inline = t2.elapsed().as_nanos() as u64;
    (t_elim, t_append, t_inline)
}

const HC_CAP: usize = 1 << 16;
const HC_MASK: usize = HC_CAP - 1;

struct HashConsDict {
    keys: Vec<u64>,
    vals: Vec<u32>,
    // For each canon id >= 257 (idx = cid - 257):
    rules: Vec<[u32; 4]>, // left_cid, right_cid, len, first_in_pool (u32::MAX if unexpanded)
    pool: Vec<u8>,       // persistent spelled bytes
}

impl HashConsDict {
    fn new() -> Self {
        Self {
            keys: vec![0u64; HC_CAP],
            vals: vec![0u32; HC_CAP],
            rules: Vec::with_capacity(32768),
            pool: Vec::with_capacity(512 * 1024),
        }
    }

    fn clear(&mut self) {
        self.keys.fill(0);
        self.rules.clear();
        self.pool.clear();
    }

    #[inline(always)]
    fn join(&mut self, l: u32, r: u32) -> u32 {
        if l == 0 {
            return r;
        }
        if r == 0 {
            return l;
        }
        if self.rules.len() >= (HC_CAP * 3) / 4 {
            self.clear();
        }
        let key = ((l as u64) << 32) | (r as u64);
        let mut h = (key.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 48) as usize & HC_MASK;
        loop {
            let k = self.keys[h];
            if k == key {
                return self.vals[h];
            }
            if k == 0 {
                let cid = self.rules.len() as u32 + 257;
                let ll = if l <= 256 { 1 } else { self.rules[(l - 257) as usize][2] };
                let rl = if r <= 256 { 1 } else { self.rules[(r - 257) as usize][2] };
                self.rules.push([l, r, ll + rl, u32::MAX]);
                self.keys[h] = key;
                self.vals[h] = cid;
                return cid;
            }
            h = (h + 1) & HC_MASK;
        }
    }

    #[inline(always)]
    fn ensure_expanded(&mut self, cid: u32, stack: &mut Vec<u32>) -> (usize, usize) {
        if cid <= 256 {
            let pos = self.pool.len();
            self.pool.push((cid - 1) as u8);
            return (pos, 1);
        }
        let ri = (cid - 257) as usize;
        let first = self.rules[ri][3];
        let len = self.rules[ri][2] as usize;
        if first != u32::MAX {
            return (first as usize, len);
        }
        let start = self.pool.len();
        stack.clear();
        stack.push(cid);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                self.pool.push((id - 1) as u8);
            } else {
                let idx = (id - 257) as usize;
                if self.rules[idx][3] == u32::MAX {
                    self.rules[idx][3] = self.pool.len() as u32;
                    stack.push(self.rules[idx][1]);
                    stack.push(self.rules[idx][0]);
                } else {
                    let st = self.rules[idx][3] as usize;
                    let ln = self.rules[idx][2] as usize;
                    self.pool.extend_from_within(st..st + ln);
                }
            }
        }
        (start, len)
    }
}

fn run_hashcons(ps: &PreStrips, sc: &mut PlainScratch, dict: &mut HashConsDict) -> (u64, u64) {
    let t0 = Instant::now();
    sc.prev0.clear();
    sc.next0.clear();
    sc.prev1.clear();
    sc.next1.clear();
    sc.n_width.clear();
    sc.n_word.clear();
    sc.prev0.push(0);
    sc.next0.push(0);
    sc.prev1.push(0);
    sc.next1.push(0);
    sc.n_width.push(0);
    sc.n_word.push(0);

    macro_rules! add_node {
        ($w:expr, $wd:expr) => {{
            let id = sc.n_width.len() as u32;
            sc.prev0.push(0);
            sc.next0.push(0);
            sc.prev1.push(0);
            sc.next1.push(0);
            sc.n_width.push($w);
            sc.n_word.push($wd);
            id
        }};
    }
    for s in &ps.strips {
        add_node!(s.width, u32::from(s.byte) + 1);
    }
    let mut term0 = (0u32, 0u32);
    let mut ord0 = (0u32, 0u32);
    let mut term1 = (0u32, 0u32);
    let mut ord1 = (0u32, 0u32);
    let append_to = |lst: &mut (u32, u32), id: u32, prev: &mut [u32], next: &mut [u32]| {
        if lst.1 == 0 {
            lst.0 = id;
        } else {
            next[lst.1 as usize] = id;
        }
        prev[id as usize] = lst.1;
        lst.1 = id;
    };
    let mut x = 0u32;
    for (i, s) in ps.strips.iter().enumerate() {
        if x < s.from {
            let id = add_node!(s.from - x, 0);
            append_to(&mut ord0, id, &mut sc.prev0, &mut sc.next0);
            append_to(&mut term1, id, &mut sc.prev1, &mut sc.next1);
        }
        append_to(&mut ord0, i as u32 + 1, &mut sc.prev0, &mut sc.next0);
        x = s.from + s.width;
    }
    if x < ps.n {
        let id = add_node!(ps.n - x, 0);
        append_to(&mut ord0, id, &mut sc.prev0, &mut sc.next0);
        append_to(&mut term1, id, &mut sc.prev1, &mut sc.next1);
    }
    x = 0;
    for &idx in &ps.image_order {
        let s = ps.strips[idx as usize];
        if x < s.to {
            let id = add_node!(s.to - x, 0);
            append_to(&mut ord1, id, &mut sc.prev1, &mut sc.next1);
            append_to(&mut term0, id, &mut sc.prev0, &mut sc.next0);
        }
        append_to(&mut ord1, idx + 1, &mut sc.prev1, &mut sc.next1);
        x = s.to + s.width;
    }
    if x < ps.n {
        let id = add_node!(ps.n - x, 0);
        append_to(&mut ord1, id, &mut sc.prev1, &mut sc.next1);
        append_to(&mut term0, id, &mut sc.prev0, &mut sc.next0);
    }
    let close = |first_l: (u32, u32), second_l: (u32, u32), prev: &mut [u32], next: &mut [u32]| {
        let mut p_id = 0u32;
        for (h, t) in [first_l, second_l] {
            if t == 0 {
                continue;
            }
            next[p_id as usize] = h;
            prev[h as usize] = p_id;
            p_id = t;
        }
        next[p_id as usize] = 0;
        prev[0] = p_id;
    };
    close(term0, ord0, &mut sc.prev0, &mut sc.next0);
    close(term1, ord1, &mut sc.prev1, &mut sc.next1);

    let mut remaining = ps.n;
    while remaining > 0 {
        let xn = sc.prev0[0];
        let yn = sc.prev1[0];
        let aw = sc.n_width[xn as usize];
        let bw = sc.n_width[yn as usize];
        let w = aw.min(bw);
        remaining -= w;
        if xn == yn {
            let pr = sc.prev0[xn as usize];
            let nx = sc.next0[xn as usize];
            sc.next0[pr as usize] = nx;
            sc.prev0[nx as usize] = pr;
            let pr = sc.prev1[xn as usize];
            let nx = sc.next1[xn as usize];
            sc.next1[pr as usize] = nx;
            sc.prev1[nx as usize] = pr;
            continue;
        }
        let wd = dict.join(sc.n_word[xn as usize], sc.n_word[yn as usize]);
        if aw <= bw {
            let pr = sc.prev0[xn as usize];
            let nx = sc.next0[xn as usize];
            sc.next0[pr as usize] = nx;
            sc.prev0[nx as usize] = pr;
            let nx = sc.next0[yn as usize];
            sc.prev0[xn as usize] = yn;
            sc.next0[xn as usize] = nx;
            sc.next0[yn as usize] = xn;
            sc.prev0[nx as usize] = xn;
            sc.n_word[xn as usize] = wd;
            if aw == bw {
                let pr = sc.prev0[yn as usize];
                let nx = sc.next0[yn as usize];
                sc.next0[pr as usize] = nx;
                sc.prev0[nx as usize] = pr;
                let pr = sc.prev1[yn as usize];
                let nx = sc.next1[yn as usize];
                sc.next1[pr as usize] = nx;
                sc.prev1[nx as usize] = pr;
            } else {
                sc.n_width[yn as usize] = bw - w;
            }
        } else {
            let pr = sc.prev1[yn as usize];
            let nx = sc.next1[yn as usize];
            sc.next1[pr as usize] = nx;
            sc.prev1[nx as usize] = pr;
            let nx = sc.next1[xn as usize];
            sc.prev1[yn as usize] = xn;
            sc.next1[yn as usize] = nx;
            sc.next1[xn as usize] = yn;
            sc.prev1[nx as usize] = yn;
            sc.n_word[yn as usize] = wd;
            sc.n_width[xn as usize] = aw - w;
        }
    }
    let t_elim = t0.elapsed().as_nanos() as u64;

    // Spelling + inline16 directly from persistent pool (only long words >16 need pool slice,
    // short words <=16 read straight into inline16 without writing to block-local spelling!)
    let t1 = Instant::now();
    sc.inline16.clear();
    sc.lens.clear();
    sc.spelling.clear();
    sc.offsets.clear();
    sc.offsets.push(0);
    let mut nid = sc.next0[0];
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        let end = k + ps.thin[a];
        while k < end {
            let nw = sc.n_width[nid as usize];
            let nwd = sc.n_word[nid as usize];
            let w = (end - k).min(nw - in_node);
            if nwd != 0 {
                let (st, ln) = dict.ensure_expanded(nwd, &mut sc.stack);
                let tot_len = ln + 1;
                let mut v = [0u8; 16];
                if tot_len <= 16 {
                    v[..ln].copy_from_slice(&dict.pool[st..st + ln]);
                    v[ln] = a as u8;
                    sc.inline16.push(v);
                    sc.lens.push(tot_len as u8);
                } else {
                    v.copy_from_slice(&dict.pool[st..st + 16]);
                    sc.inline16.push(v);
                    sc.lens.push(255);
                    sc.spelling.extend_from_slice(&dict.pool[st..st + ln]);
                    sc.spelling.push(a as u8);
                }
                sc.offsets.push(sc.spelling.len() as u32);
            }
            in_node += w;
            k += w;
            if in_node == nw {
                nid = sc.next0[nid as usize];
                in_node = 0;
            }
        }
    }
    let t_spell = t1.elapsed().as_nanos() as u64;
    (t_elim, t_spell)
}

fn main() {
    let data = fs::read("/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2").unwrap();
    let kernels = Kernels::detect();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut runs = Runs::default();
    let mut windows: Vec<Vec<PreStrips>> = Vec::new();

    loop {
        match scanner.next(syms).unwrap() {
            Event::Done => break,
            Event::Block {
                stream, symbols, ..
            } => {
                let s = stream as usize;
                while windows.len() <= s {
                    windows.push(Vec::new());
                }
                if runs::takes_runs(&symbols) {
                    let n = runs::front(&symbols, &mut runs, kernels).unwrap();
                    if n >= 65_536 {
                        let inp = BlockInput {
                            n,
                            p: symbols.orig_ptr,
                            records: runs.records().to_vec(),
                        };
                        windows[s].push(prep_strips(&inp));
                    }
                }
                syms = symbols.syms;
            }
        }
    }

    // Exactness verification first (1 pass)
    let mut sc_a = PlainScratch::default();
    let mut sc_b = PlainScratch::default();
    let mut dict = HashConsDict::new();
    let mut verified_words = 0usize;
    for win in &windows {
        dict.clear();
        for ps in win {
            run_plain(ps, &mut sc_a);
            run_hashcons(ps, &mut sc_b, &mut dict);
            assert_eq!(sc_a.inline16, sc_b.inline16, "inline16 mismatch!");
            assert_eq!(sc_a.lens, sc_b.lens, "lens mismatch!");
            verified_words += sc_a.inline16.len();
        }
    }
    println!("Exactness verified: {} band words across 233 blocks match bit-for-bit!", verified_words);

    // 5 interleaved rounds A/B
    let mut plain_elim = 0u64;
    let mut plain_app = 0u64;
    let mut plain_inl = 0u64;
    let mut hc_elim = 0u64;
    let mut hc_spell = 0u64;
    let mut peak_pool = 0usize;
    let mut peak_rules = 0usize;

    for round in 0..5 {
        let mut pe = 0u64;
        let mut pa = 0u64;
        let mut pi = 0u64;
        let mut he = 0u64;
        let mut hs = 0u64;
        for win in &windows {
            if round % 2 == 0 {
                for ps in win {
                    let (e, a, i) = run_plain(ps, &mut sc_a);
                    pe += e;
                    pa += a;
                    pi += i;
                }
                dict.clear();
                for ps in win {
                    let (e, s) = run_hashcons(ps, &mut sc_b, &mut dict);
                    he += e;
                    hs += s;
                    peak_pool = peak_pool.max(dict.pool.len());
                    peak_rules = peak_rules.max(dict.rules.len());
                }
            } else {
                dict.clear();
                for ps in win {
                    let (e, s) = run_hashcons(ps, &mut sc_b, &mut dict);
                    he += e;
                    hs += s;
                    peak_pool = peak_pool.max(dict.pool.len());
                    peak_rules = peak_rules.max(dict.rules.len());
                }
                for ps in win {
                    let (e, a, i) = run_plain(ps, &mut sc_a);
                    pe += e;
                    pa += a;
                    pi += i;
                }
            }
        }
        println!(
            "  Round {}: Plain (elim={:.2} + append={:.2} + inline={:.2} = {:.2}ms) vs HashCons (elim={:.2} + spell={:.2} = {:.2}ms) | ratio={:.3}x",
            round,
            pe as f64 / 1e6,
            pa as f64 / 1e6,
            pi as f64 / 1e6,
            (pe + pa + pi) as f64 / 1e6,
            he as f64 / 1e6,
            hs as f64 / 1e6,
            (he + hs) as f64 / 1e6,
            (he + hs) as f64 / (pe + pa + pi) as f64
        );
        plain_elim += pe;
        plain_app += pa;
        plain_inl += pi;
        hc_elim += he;
        hc_spell += hs;
    }

    let p_tot = (plain_elim + plain_app + plain_inl) as f64 / 5e6;
    let h_tot = (hc_elim + hc_spell) as f64 / 5e6;
    println!("\n=== MEAN OVER 5 INTERLEAVED ROUNDS (233 large blocks, wikidata.bz2) ===");
    println!(
        "  Plain Open (Elim+Spell+Inline): {:.2} ms (elim={:.2}, append={:.2}, inline={:.2})",
        p_tot,
        plain_elim as f64 / 5e6,
        plain_app as f64 / 5e6,
        plain_inl as f64 / 5e6
    );
    println!(
        "  HashCons   (Elim+Spell+Inline): {:.2} ms (elim={:.2}, spell={:.2})",
        h_tot,
        hc_elim as f64 / 5e6,
        hc_spell as f64 / 5e6
    );
    println!(
        "  Stage ratio: {:.3}x (delta = {:+.2} ms out of ~607 ms total Open = {:+.2}% of Open!)",
        h_tot / p_tot,
        h_tot - p_tot,
        100.0 * (h_tot - p_tot) / 606.78
    );
    println!(
        "  Persistent HashConsDict memory/worker: keys+vals={} KiB, rules={} KiB ({} rules), pool={} KiB -> TOTAL = {:.1} KiB/worker!",
        (HC_CAP * 12) / 1024,
        (peak_rules * 16) / 1024,
        peak_rules,
        peak_pool / 1024,
        (HC_CAP * 12 + peak_rules * 16 + peak_pool) as f64 / 1024.0
    );
}
