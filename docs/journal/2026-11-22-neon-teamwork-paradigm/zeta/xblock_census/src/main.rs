use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: u32,
    code: u32,
}

struct OpenDiag {
    out: Vec<u8>,
    n: usize,
    r: usize,
    records_len: usize,
    m: usize,
    mark: u32,
    strips: Vec<Strip>,
    bands: Vec<Band>,
    words: Vec<Vec<u8>>, // code = 256 + idx
    spelling_bytes: usize,
    steps: usize,
    num_rules: usize,
    rule_words: HashSet<Vec<u8>>,
    word_emit_counts: HashMap<Vec<u8>, usize>,
    // For each retained rank j in 0..m:
    //   ret_tpos[j] = start pos in `out` where j emits its word
    //   ret_end_tpos[j] = (ret_tpos[j] + word_len - 1) % n (text pos of thin row byte)
    //   ret_width[j] = maximal run width in L of the thin row `edges_next[j]`
    ret_end_tpos: Vec<usize>,
    ret_width: Vec<u32>,
    counts: [u32; 256],
    c_arr: [u32; 256],
    l_col: Vec<u8>,
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

fn run_open_diag(records: &[u16], n: usize, p: u32, limit: u32) -> OpenDiag {
    let n32 = n as u32;
    let mut all = [0u32; 256];
    let mut thin = [0u32; 256];
    let mut thick = [0u32; 256];
    let mut m = 0u32;
    let mut mark = 0u32;
    let mut r = 0usize;
    let mut strips: Vec<Strip> = Vec::new();
    let mut thin_pieces: Vec<(u8, u32, u32, u32)> = Vec::new();
    let mut ret_orig_width: Vec<u32> = Vec::new();
    let mut l_col: Vec<u8> = Vec::with_capacity(n);

    maximal(records, |c, pos, width| {
        r += 1;
        l_col.resize(l_col.len() + width as usize, c);
        split(c, pos, width, p, limit, |c, x, w, keep| {
            let a = usize::from(c);
            if keep {
                if x <= p && p < x + w {
                    mark = m + (p - x);
                }
                thin_pieces.push((c, x, w, m));
                ret_orig_width.resize(ret_orig_width.len() + w as usize, width);
                thin[a] += w;
                m += w;
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
    assert_eq!(l_col.len(), n);

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

    // Endpoint elimination (exact mirror of src/open/endpoint.rs, using Vec<u32> for simplicity)
    let mut prev0 = vec![0u32];
    let mut next0 = vec![0u32];
    let mut prev1 = vec![0u32];
    let mut next1 = vec![0u32];
    let mut n_width = vec![0u32];
    let mut n_word = vec![0u32];

    macro_rules! add_node {
        ($w:expr, $wd:expr) => {{
            let id = n_width.len() as u32;
            prev0.push(0);
            next0.push(0);
            prev1.push(0);
            next1.push(0);
            n_width.push($w);
            n_word.push($wd);
            id
        }};
    }

    for s in &strips {
        add_node!(s.width, u32::from(s.byte) + 1);
    }

    let mut term0 = (0u32, 0u32);
    let mut ord0 = (0u32, 0u32);
    let mut term1 = (0u32, 0u32);
    let mut ord1 = (0u32, 0u32);

    let append_to = |lst: &mut (u32, u32),
                     id: u32,
                     prev: &mut [u32],
                     next: &mut [u32]| {
        if lst.1 == 0 {
            lst.0 = id;
        } else {
            next[lst.1 as usize] = id;
        }
        prev[id as usize] = lst.1;
        lst.1 = id;
    };

    let mut x = 0u32;
    for (i, s) in strips.iter().enumerate() {
        if x < s.from {
            let id = add_node!(s.from - x, 0);
            append_to(&mut ord0, id, &mut prev0, &mut next0);
            append_to(&mut term1, id, &mut prev1, &mut next1);
        }
        append_to(&mut ord0, i as u32 + 1, &mut prev0, &mut next0);
        x = s.from + s.width;
    }
    if x < n32 {
        let id = add_node!(n32 - x, 0);
        append_to(&mut ord0, id, &mut prev0, &mut next0);
        append_to(&mut term1, id, &mut prev1, &mut next1);
    }

    x = 0;
    for &idx in &image_order {
        let s = strips[idx as usize];
        if x < s.to {
            let id = add_node!(s.to - x, 0);
            append_to(&mut ord1, id, &mut prev1, &mut next1);
            append_to(&mut term0, id, &mut prev0, &mut next0);
        }
        append_to(&mut ord1, idx + 1, &mut prev1, &mut next1);
        x = s.to + s.width;
    }
    if x < n32 {
        let id = add_node!(n32 - x, 0);
        append_to(&mut ord1, id, &mut prev1, &mut next1);
        append_to(&mut term0, id, &mut prev0, &mut next0);
    }

    let close = |first_l: (u32, u32),
                 second_l: (u32, u32),
                 prev: &mut [u32],
                 next: &mut [u32]| {
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

    close(term0, ord0, &mut prev0, &mut next0);
    close(term1, ord1, &mut prev1, &mut next1);

    let mut rules: Vec<(u32, u32, u32)> = Vec::new();
    let wlen = |wd: u32, rules: &[(u32, u32, u32)]| -> u32 {
        if wd <= 256 {
            u32::from(wd != 0)
        } else {
            rules[(wd - 257) as usize].2
        }
    };
    let join = |l: u32, r_id: u32, rules: &mut Vec<(u32, u32, u32)>| -> u32 {
        if l == 0 {
            return r_id;
        }
        if r_id == 0 {
            return l;
        }
        let id = rules.len() as u32 + 257;
        let len = wlen(l, rules) + wlen(r_id, rules);
        rules.push((l, r_id, len));
        id
    };

    let cut = |id: u32, prev: &mut [u32], next: &mut [u32]| {
        let pr = prev[id as usize];
        let nx = next[id as usize];
        next[pr as usize] = nx;
        prev[nx as usize] = pr;
    };
    let insert = |at: u32, id: u32, prev: &mut [u32], next: &mut [u32]| {
        let nx = next[at as usize];
        prev[id as usize] = at;
        next[id as usize] = nx;
        next[at as usize] = id;
        prev[nx as usize] = id;
    };

    let mut remaining = n32;
    let mut steps = 0usize;
    while remaining > 0 {
        let xn = prev0[0];
        let yn = prev1[0];
        let aw = n_width[xn as usize];
        let bw = n_width[yn as usize];
        let w = aw.min(bw);
        remaining -= w;
        steps += 1;
        if xn == yn {
            cut(xn, &mut prev0, &mut next0);
            cut(xn, &mut prev1, &mut next1);
            continue;
        }
        let wd = join(n_word[xn as usize], n_word[yn as usize], &mut rules);
        if aw <= bw {
            cut(xn, &mut prev0, &mut next0);
            insert(yn, xn, &mut prev0, &mut next0);
            n_word[xn as usize] = wd;
            if aw == bw {
                cut(yn, &mut prev0, &mut next0);
                cut(yn, &mut prev1, &mut next1);
            } else {
                n_width[yn as usize] = bw - w;
            }
        } else {
            cut(yn, &mut prev1, &mut next1);
            insert(xn, yn, &mut prev1, &mut next1);
            n_word[yn as usize] = wd;
            n_width[xn as usize] = aw - w;
        }
    }

    let mut bottom = vec![0u32; n_width.len()];
    let mut nid = next1[0];
    x = 0;
    while nid != 0 {
        bottom[nid as usize] = x;
        x += n_width[nid as usize];
        nid = next1[nid as usize];
    }
    assert_eq!(x, m);

    // Memoized expansion of rules
    let mut memo: Vec<Option<Vec<u8>>> = vec![None; rules.len()];
    let mut stack: Vec<u32> = Vec::new();
    let mut order: Vec<u32> = Vec::new();
    let mut expand_word = |wd: u32, dest: &mut Vec<u8>| {
        if wd == 0 {
            return;
        }
        if wd <= 256 {
            dest.push((wd - 1) as u8);
            return;
        }
        let idx = (wd - 257) as usize;
        if memo[idx].is_none() {
            stack.clear();
            order.clear();
            stack.push(wd);
            while let Some(cur) = stack.pop() {
                if cur <= 256 {
                    continue;
                }
                let ci = (cur - 257) as usize;
                if memo[ci].is_some() {
                    continue;
                }
                order.push(cur);
                let (l, r_id, _) = rules[ci];
                if l > 256 && memo[(l - 257) as usize].is_none() {
                    stack.push(l);
                }
                if r_id > 256 && memo[(r_id - 257) as usize].is_none() {
                    stack.push(r_id);
                }
            }
            order.sort_unstable();
            order.dedup();
            for &cur in &order {
                let ci = (cur - 257) as usize;
                if memo[ci].is_some() {
                    continue;
                }
                let (l, r_id, len) = rules[ci];
                let mut v = Vec::with_capacity(len as usize);
                if l <= 256 {
                    v.push((l - 1) as u8);
                } else {
                    v.extend_from_slice(memo[(l - 257) as usize].as_ref().unwrap());
                }
                if r_id <= 256 {
                    v.push((r_id - 1) as u8);
                } else {
                    v.extend_from_slice(memo[(r_id - 257) as usize].as_ref().unwrap());
                }
                memo[ci] = Some(v);
            }
        }
        dest.extend_from_slice(memo[idx].as_ref().unwrap());
    };

    let mut cursors = [0usize; 256];
    let mut bands: Vec<Band> = Vec::new();
    let mut words: Vec<Vec<u8>> = Vec::new();
    let mut spelling_bytes = 0usize;

    nid = next0[0];
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        cursors[a] = bands.len();
        let end = k + thin[a];
        while k < end {
            let nw = n_width[nid as usize];
            let nwd = n_word[nid as usize];
            let w = (end - k).min(nw - in_node);
            let dst = bottom[nid as usize] + in_node;
            let code = if nwd == 0 {
                a as u32
            } else {
                let mut wb = Vec::new();
                expand_word(nwd, &mut wb);
                wb.push(a as u8);
                spelling_bytes += wb.len();
                let c_id = 256 + words.len() as u32;
                words.push(wb);
                c_id
            };
            bands.push(Band {
                end: k + w,
                delta: dst.wrapping_sub(k),
                code,
            });
            in_node += w;
            k += w;
            if in_node == nw {
                nid = next0[nid as usize];
                in_node = 0;
            }
        }
    }
    assert_eq!(nid, 0);

    // Wire edges
    let mut stream = starts(thin);
    let mut edges_next = vec![0u32; m as usize];
    let mut edges_code = vec![0u32; m as usize];
    let mut cur_band = cursors;
    let mut j = 0u32;
    for &(c, _, w, _) in &thin_pieces {
        let a = usize::from(c);
        let mut z = stream[a];
        let end = z + w;
        while z < end {
            let b = bands[cur_band[a]];
            let span = end.min(b.end) - z;
            let dest = z.wrapping_add(b.delta) as usize;
            for off in 0..span as usize {
                edges_next[dest + off] = j + off as u32;
                edges_code[dest + off] = b.code;
            }
            j += span;
            z += span;
            if z == b.end {
                cur_band[a] += 1;
            }
        }
        stream[a] = end;
    }

    // Emit walk
    let mut out = Vec::with_capacity(n);
    let mut word_emit_counts: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut ret_end_tpos: Vec<usize> = Vec::with_capacity(m as usize);
    let mut ret_width: Vec<u32> = Vec::with_capacity(m as usize);
    let mut cur = mark as usize;
    for _ in 0..m {
        let code = edges_code[cur];
        let nxt = edges_next[cur] as usize;
        let tpos = out.len();
        if code < 256 {
            let b = code as u8;
            out.push(b);
            *word_emit_counts.entry(vec![b]).or_insert(0) += 1;
            ret_end_tpos.push(tpos % n);
        } else {
            let wb = &words[(code - 256) as usize];
            out.extend_from_slice(wb);
            *word_emit_counts.entry(wb.clone()).or_insert(0) += 1;
            ret_end_tpos.push((tpos + wb.len() - 1) % n);
        }
        ret_width.push(ret_orig_width[nxt]);
        cur = nxt;
        if cur == mark as usize || out.len() >= n {
            break;
        }
    }
    if out.len() < n {
        let period = out.clone();
        while out.len() < n {
            let take = (n - out.len()).min(period.len());
            out.extend_from_slice(&period[..take]);
        }
    }
    out.truncate(n);

    let mut rule_words: HashSet<Vec<u8>> = HashSet::new();
    for opt in memo {
        if let Some(v) = opt {
            rule_words.insert(v);
        }
    }

    OpenDiag {
        out,
        n,
        r,
        records_len: records.len(),
        m: m as usize,
        mark,
        strips,
        bands,
        words,
        spelling_bytes,
        steps,
        num_rules: rules.len(),
        rule_words,
        word_emit_counts,
        ret_end_tpos,
        ret_width,
        counts: all,
        c_arr: first,
        l_col,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TokCls {
    Other = 0,
    Guid = 1,
    Sha1 = 2,
    Entity = 3,
    JsonKey = 4,
}

fn is_hex(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

fn is_low_hex(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'a'..=b'f')
}

struct TokenIndex {
    // For each byte position: (TokCls, token slice range (start, end))
    pos_cls: Vec<u8>,
    pos_tok_range: Vec<(u32, u32)>,
    guids: HashSet<Vec<u8>>,
    sha1s: HashMap<Vec<u8>, usize>,
    entities: HashMap<Vec<u8>, usize>,
    keys: HashSet<Vec<u8>>,
}

fn classify_tokens(text: &[u8]) -> TokenIndex {
    let n = text.len();
    let mut pos_cls = vec![0u8; n];
    let mut pos_tok_range = vec![(0u32, 0u32); n];
    let mut guids = HashSet::new();
    let mut sha1s: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut entities: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut keys = HashSet::new();

    // 1. GUIDs: 8-4-4-4-12 hex
    let mut i = 0;
    while i + 36 <= n {
        if text[i + 8] == b'-'
            && text[i + 13] == b'-'
            && text[i + 18] == b'-'
            && text[i + 23] == b'-'
        {
            let mut ok = true;
            for (k, &b) in text[i..i + 36].iter().enumerate() {
                if k == 8 || k == 13 || k == 18 || k == 23 {
                    continue;
                }
                if !is_hex(b) {
                    ok = false;
                    break;
                }
            }
            if ok {
                guids.insert(text[i..i + 36].to_vec());
                for p in i..i + 36 {
                    pos_cls[p] = TokCls::Guid as u8;
                    pos_tok_range[p] = (i as u32, (i + 36) as u32);
                }
                i += 36;
                continue;
            }
        }
        i += 1;
    }

    // 2a. sha1: exact `"<40 lowercase hex>"` (cannot drift quote parity!)
    i = 0;
    while i + 42 <= n {
        if text[i] == b'"'
            && text[i + 41] == b'"'
            && pos_cls[i + 1] == 0
            && text[i + 1..i + 41].iter().all(|&b| is_low_hex(b))
        {
            *sha1s.entry(text[i + 1..i + 41].to_vec()).or_insert(0) += 1;
            for p in i + 1..i + 41 {
                pos_cls[p] = TokCls::Sha1 as u8;
                pos_tok_range[p] = ((i + 1) as u32, (i + 41) as u32);
            }
            i += 42;
            continue;
        }
        i += 1;
    }

    // 2b. json_key: anchor on `":` and scan backward up to 64 chars to opening `"`
    i = 2;
    while i + 1 < n {
        if text[i] == b'"' && text[i + 1] == b':' {
            let mut s = i;
            let lo = i.saturating_sub(64);
            let mut found = None;
            while s > lo {
                s -= 1;
                if text[s] == b'"' {
                    found = Some(s + 1);
                    break;
                }
                if text[s] == b'\\' || text[s] == b'\n' || text[s] == b'{' || text[s] == b',' {
                    break;
                }
            }
            if let Some(start) = found {
                if start < i && pos_cls[start] == 0 {
                    keys.insert(text[start..i].to_vec());
                    for p in start..i {
                        if pos_cls[p] == 0 {
                            pos_cls[p] = TokCls::JsonKey as u8;
                            pos_tok_range[p] = (start as u32, i as u32);
                        }
                    }
                }
            }
        }
        i += 1;
    }

    // 3. Entity IDs: Q/P/L + 1..10 digits with word boundary
    i = 0;
    while i + 2 <= n {
        if pos_cls[i] == 0 && matches!(text[i], b'Q' | b'P' | b'L') && text[i + 1].is_ascii_digit()
        {
            let prev_ok = i == 0 || !text[i - 1].is_ascii_alphanumeric();
            if prev_ok {
                let mut end = i + 1;
                while end < n && text[end].is_ascii_digit() && end - i <= 11 {
                    end += 1;
                }
                let next_ok = end == n || !text[end].is_ascii_alphanumeric();
                if next_ok && end - i <= 11 {
                    *entities.entry(text[i..end].to_vec()).or_insert(0) += 1;
                    for p in i..end {
                        if pos_cls[p] == 0 {
                            pos_cls[p] = TokCls::Entity as u8;
                            pos_tok_range[p] = (i as u32, end as u32);
                        }
                    }
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
    }

    TokenIndex {
        pos_cls,
        pos_tok_range,
        guids,
        sha1s,
        entities,
        keys,
    }
}

struct RankIndex {
    step: usize,
    chk: Vec<[u32; 256]>,
    c: [u32; 256],
}

impl RankIndex {
    fn new(l: &[u8], c: [u32; 256]) -> Self {
        let step = 1024;
        let mut chk = Vec::with_capacity(l.len() / step + 2);
        let mut cur = [0u32; 256];
        for chunk in l.chunks(step) {
            chk.push(cur);
            for &b in chunk {
                cur[b as usize] += 1;
            }
        }
        chk.push(cur);
        RankIndex { step, chk, c }
    }

    fn lf_step(&self, l: &[u8], byte: u8, pos: usize) -> usize {
        let blk = pos / self.step;
        let mut cnt = self.chk[blk][byte as usize] as usize;
        for &b in &l[blk * self.step..pos] {
            cnt += usize::from(b == byte);
        }
        self.c[byte as usize] as usize + cnt
    }

    fn bsearch(&self, l: &[u8], w: &[u8]) -> (usize, usize) {
        let (mut lo, mut hi) = (0usize, l.len());
        for &b in w.iter().rev() {
            lo = self.lf_step(l, b, lo);
            hi = self.lf_step(l, b, hi);
            if lo >= hi {
                return (lo, hi);
            }
        }
        (lo, hi)
    }

    fn lf_at(&self, l: &[u8], pos: usize) -> usize {
        let b = l[pos];
        self.lf_step(l, b, pos)
    }
}

struct BlockRec {
    stream: usize,
    in_stream: usize,
    global_idx: usize,
    diag: OpenDiag,
    toks: TokenIndex,
    admitted: bool,
}

fn check_admitted(records: usize, n: usize, m: usize, strips: usize) -> bool {
    if n == 0 || n > 900_000 {
        return false;
    }
    // LazyCells admit: n >= 64 && records*64 >= n && records*20 < n*3 (i.e. 1/64 <= R/n < 0.15)
    let lazy_admit = records * 64 >= n && records * 20 < n * 3;
    let dense_admit = records * 64 >= n
        && records * 25 < n * 7
        && m <= 256_000.min(n * 3 / 10)
        && strips <= 16_384;
    lazy_admit || dense_admit
}

#[derive(Default)]
struct PairAgg {
    pairs: usize,
    // Open words
    sum_words_b_distinct: usize,
    sum_words_shared_distinct: usize,
    sum_spelling_b: usize,
    sum_spelling_shared: usize,
    sum_m16: usize,
    sum_nonlit_rows: usize,
    sum_nonlit_rows_shared: usize,
    sum_bytes_n: usize,
    sum_bytes_shared_open: usize,
    sum_rules_b: usize,
    sum_rules_shared: usize,
    // Token retention
    sum_ret_cls: [usize; 5],
    sum_ret_cls_in_a: [usize; 5],
    sum_ret_w1: usize,
    sum_ret_w1_in_a: [usize; 5],
    sum_sha1_b_occ: usize,
    sum_sha1_shared_occ: usize,
    sum_sha1_b_dist: usize,
    sum_sha1_shared_dist: usize,
    sum_ent_b_occ: usize,
    sum_ent_shared_occ: usize,
    // k-gram coverage (k=8, 16, 32)
    sum_k8_ret_cov: usize,
    sum_k16_ret_cov: usize,
    sum_k32_ret_cov: usize,
    sum_k8_w1_cov: usize,
    sum_k16_w1_cov: usize,
    sum_k32_w1_cov: usize,
    // BWT stability
    sum_tv_freq: f64,
    sum_bwt_sample: usize,
    sum_bwt_exact_w: usize,
    sum_bwt_exact_delta: usize,
    bwt_row_drifts: Vec<usize>,
    bwt_delta_drifts: Vec<usize>,
}

fn eval_pair(
    a: &BlockRec,
    b: &BlockRec,
    agg: &mut PairAgg,
    do_kgrams: bool,
    do_bwt_stab: bool,
) {
    agg.pairs += 1;
    let words_a: HashSet<&[u8]> = a.diag.words.iter().map(|v| v.as_slice()).collect();
    let words_b_dist: HashSet<&[u8]> = b.diag.words.iter().map(|v| v.as_slice()).collect();
    let shared_dist = words_b_dist.iter().filter(|w| words_a.contains(**w)).count();
    let shared_spell: usize = b
        .diag
        .words
        .iter()
        .filter(|w| words_a.contains(w.as_slice()))
        .map(|w| w.len())
        .sum();

    let mut nonlit_rows = 0usize;
    let mut nonlit_rows_shared = 0usize;
    let mut bytes_shared = 0usize;
    for (w, &cnt) in &b.diag.word_emit_counts {
        if w.len() >= 2 {
            nonlit_rows += cnt;
            if words_a.contains(w.as_slice()) {
                nonlit_rows_shared += cnt;
                bytes_shared += w.len() * cnt;
            }
        }
    }

    let rules_shared = b
        .diag
        .rule_words
        .iter()
        .filter(|w| a.diag.rule_words.contains(*w))
        .count();

    agg.sum_words_b_distinct += words_b_dist.len();
    agg.sum_words_shared_distinct += shared_dist;
    agg.sum_spelling_b += b.diag.spelling_bytes;
    agg.sum_spelling_shared += shared_spell;
    agg.sum_m16 += b.diag.m;
    agg.sum_nonlit_rows += nonlit_rows;
    agg.sum_nonlit_rows_shared += nonlit_rows_shared;
    agg.sum_bytes_n += b.diag.n;
    agg.sum_bytes_shared_open += bytes_shared;
    agg.sum_rules_b += b.diag.rule_words.len();
    agg.sum_rules_shared += rules_shared;

    // Token retention
    for (&epos, &w) in b.diag.ret_end_tpos.iter().zip(b.diag.ret_width.iter()) {
        let cls = b.toks.pos_cls[epos] as usize;
        agg.sum_ret_cls[cls] += 1;
        let (s, e) = b.toks.pos_tok_range[epos];
        let in_a = match cls {
            1 => a.toks.guids.contains(&b.diag.out[s as usize..e as usize]),
            2 => a.toks.sha1s.contains_key(&b.diag.out[s as usize..e as usize]),
            3 => a.toks.entities.contains_key(&b.diag.out[s as usize..e as usize]),
            4 => a.toks.keys.contains(&b.diag.out[s as usize..e as usize]),
            _ => false,
        };
        if in_a {
            agg.sum_ret_cls_in_a[cls] += 1;
        }
        if w == 1 {
            agg.sum_ret_w1 += 1;
            if in_a {
                agg.sum_ret_w1_in_a[cls] += 1;
            }
        }
    }

    // sha1 & entity details
    for (tok, &cnt) in &b.toks.sha1s {
        agg.sum_sha1_b_occ += cnt;
        agg.sum_sha1_b_dist += 1;
        if a.toks.sha1s.contains_key(tok) {
            agg.sum_sha1_shared_occ += cnt;
            agg.sum_sha1_shared_dist += 1;
        }
    }
    for (tok, &cnt) in &b.toks.entities {
        agg.sum_ent_b_occ += cnt;
        if a.toks.entities.contains_key(tok) {
            agg.sum_ent_shared_occ += cnt;
        }
    }

    if do_kgrams {
        // Exact k-gram coverage of retained rows and w=1 rows for k=8, 16, 32
        for &k in &[8usize, 16, 32] {
            let mut grams_a: HashSet<&[u8]> = HashSet::with_capacity(a.diag.n);
            for w in a.diag.out.windows(k) {
                grams_a.insert(w);
            }
            let mut cov = vec![0u8; b.diag.n];
            let mut last_end = 0usize;
            for (i, w) in b.diag.out.windows(k).enumerate() {
                if grams_a.contains(w) {
                    let s = i.max(last_end);
                    let e = i + k;
                    if e > s {
                        cov[s..e].fill(1);
                        last_end = e;
                    }
                }
            }
            let mut rc = 0usize;
            let mut wc = 0usize;
            for (&epos, &w) in b.diag.ret_end_tpos.iter().zip(b.diag.ret_width.iter()) {
                if cov[epos] != 0 {
                    rc += 1;
                    if w == 1 {
                        wc += 1;
                    }
                }
            }
            match k {
                8 => {
                    agg.sum_k8_ret_cov += rc;
                    agg.sum_k8_w1_cov += wc;
                }
                16 => {
                    agg.sum_k16_ret_cov += rc;
                    agg.sum_k16_w1_cov += wc;
                }
                32 => {
                    agg.sum_k32_ret_cov += rc;
                    agg.sum_k32_w1_cov += wc;
                }
                _ => {}
            }
        }
    }

    if do_bwt_stab {
        let mut tv = 0.0f64;
        for c in 0..256 {
            let fa = a.diag.counts[c] as f64 / a.diag.n as f64;
            let fb = b.diag.counts[c] as f64 / b.diag.n as f64;
            tv += (fa - fb).abs();
        }
        agg.sum_tv_freq += 0.5 * tv;

        let ra = RankIndex::new(&a.diag.l_col, a.diag.c_arr);
        let rb = RankIndex::new(&b.diag.l_col, b.diag.c_arr);
        let mut shared_long: Vec<&[u8]> = words_b_dist
            .iter()
            .copied()
            .filter(|w| w.len() >= 6 && words_a.contains(*w))
            .collect();
        shared_long.sort_unstable_by(|x, y| y.len().cmp(&x.len()).then(x.cmp(y)));
        for &w in shared_long.iter().take(100) {
            let (la, ha) = ra.bsearch(&a.diag.l_col, w);
            let (lb, hb) = rb.bsearch(&b.diag.l_col, w);
            if la < ha && lb < hb {
                agg.sum_bwt_sample += 1;
                agg.bwt_row_drifts.push(la.abs_diff(lb));
                if (ha - la) == (hb - lb) {
                    agg.sum_bwt_exact_w += 1;
                }
                let da = ra.lf_at(&a.diag.l_col, la) as isize - la as isize;
                let db = rb.lf_at(&b.diag.l_col, lb) as isize - lb as isize;
                if da == db {
                    agg.sum_bwt_exact_delta += 1;
                }
                agg.bwt_delta_drifts.push(da.abs_diff(db));
            }
        }
    }
}

fn print_agg(title: &str, agg: &mut PairAgg, has_kgrams: bool, has_bwt: bool) {
    println!("\n=== {} ({} pairs) ===", title, agg.pairs);
    if agg.pairs == 0 {
        return;
    }
    println!(
        "  Open distinct non-lit words shared:  {}/{} ({:.2}%)",
        agg.sum_words_shared_distinct,
        agg.sum_words_b_distinct,
        100.0 * agg.sum_words_shared_distinct as f64 / agg.sum_words_b_distinct.max(1) as f64
    );
    println!(
        "  Open spelling bytes shared:          {}/{} B ({:.2}%)",
        agg.sum_spelling_shared,
        agg.sum_spelling_b,
        100.0 * agg.sum_spelling_shared as f64 / agg.sum_spelling_b.max(1) as f64
    );
    println!(
        "  Open non-lit rows shared:            {}/{} ({:.2}% of non-lit, {:.2}% of all m16={})",
        agg.sum_nonlit_rows_shared,
        agg.sum_nonlit_rows,
        100.0 * agg.sum_nonlit_rows_shared as f64 / agg.sum_nonlit_rows.max(1) as f64,
        100.0 * agg.sum_nonlit_rows_shared as f64 / agg.sum_m16.max(1) as f64,
        agg.sum_m16
    );
    println!(
        "  Emitted bytes from Open(A) words:    {}/{} B ({:.2}% of n)",
        agg.sum_bytes_shared_open,
        agg.sum_bytes_n,
        100.0 * agg.sum_bytes_shared_open as f64 / agg.sum_bytes_n.max(1) as f64
    );
    println!(
        "  RV SLP internal rule strings shared: {}/{} ({:.2}%)",
        agg.sum_rules_shared,
        agg.sum_rules_b,
        100.0 * agg.sum_rules_shared as f64 / agg.sum_rules_b.max(1) as f64
    );
    let names = ["other", "GUID", "sha1", "entity_id", "json_key"];
    println!("  Retained rows m16 by token class (shared with prev block A):");
    for c in [1, 2, 3, 4, 0] {
        let tot = agg.sum_ret_cls[c];
        let sh = agg.sum_ret_cls_in_a[c];
        let w1sh = agg.sum_ret_w1_in_a[c];
        println!(
            "    {:10}: {:8} ({:5.2}% m16) | tok in A: {:8} ({:5.2}% of cls, {:5.2}% m16) | w=1 in A: {:7}",
            names[c],
            tot,
            100.0 * tot as f64 / agg.sum_m16.max(1) as f64,
            sh,
            100.0 * sh as f64 / tot.max(1) as f64,
            100.0 * sh as f64 / agg.sum_m16.max(1) as f64,
            w1sh
        );
    }
    let named_in_a: usize = agg.sum_ret_cls_in_a[1..=4].iter().sum();
    let named_w1_in_a: usize = agg.sum_ret_w1_in_a[1..=4].iter().sum();
    println!(
        "    TOTAL named tokens in A: {} / {} m16 ({:.2}%), w=1: {} / {} ({:.2}%)",
        named_in_a,
        agg.sum_m16,
        100.0 * named_in_a as f64 / agg.sum_m16.max(1) as f64,
        named_w1_in_a,
        agg.sum_ret_w1,
        100.0 * named_w1_in_a as f64 / agg.sum_ret_w1.max(1) as f64
    );
    println!(
        "  sha1 overlap: distinct {}/{} ({:.2}%), occurrences {}/{} ({:.2}%)",
        agg.sum_sha1_shared_dist,
        agg.sum_sha1_b_dist,
        100.0 * agg.sum_sha1_shared_dist as f64 / agg.sum_sha1_b_dist.max(1) as f64,
        agg.sum_sha1_shared_occ,
        agg.sum_sha1_b_occ,
        100.0 * agg.sum_sha1_shared_occ as f64 / agg.sum_sha1_b_occ.max(1) as f64
    );
    println!(
        "  entity_id occurrences from A: {}/{} ({:.2}%)",
        agg.sum_ent_shared_occ,
        agg.sum_ent_b_occ,
        100.0 * agg.sum_ent_shared_occ as f64 / agg.sum_ent_b_occ.max(1) as f64
    );
    if has_kgrams {
        println!(
            "  Oracle k-gram coverage of m16 / w=1: k=8: {:.2}% / {:.2}%, k=16: {:.2}% / {:.2}%, k=32: {:.2}% / {:.2}%",
            100.0 * agg.sum_k8_ret_cov as f64 / agg.sum_m16.max(1) as f64,
            100.0 * agg.sum_k8_w1_cov as f64 / agg.sum_ret_w1.max(1) as f64,
            100.0 * agg.sum_k16_ret_cov as f64 / agg.sum_m16.max(1) as f64,
            100.0 * agg.sum_k16_w1_cov as f64 / agg.sum_ret_w1.max(1) as f64,
            100.0 * agg.sum_k32_ret_cov as f64 / agg.sum_m16.max(1) as f64,
            100.0 * agg.sum_k32_w1_cov as f64 / agg.sum_ret_w1.max(1) as f64,
        );
    }
    if has_bwt && !agg.bwt_row_drifts.is_empty() {
        agg.bwt_row_drifts.sort_unstable();
        agg.bwt_delta_drifts.sort_unstable();
        let m_row = agg.bwt_row_drifts[agg.bwt_row_drifts.len() / 2];
        let m_del = agg.bwt_delta_drifts[agg.bwt_delta_drifts.len() / 2];
        println!(
            "  BWT/LF stability (sample={} shared len>=6 words): mean TV_freq={:.4}, median |lA-lB|={}, exact_width={}/{} ({:.2}%), exact_LF_delta={}/{} ({:.2}%), median |dA-dB|={}",
            agg.sum_bwt_sample,
            agg.sum_tv_freq / agg.pairs as f64,
            m_row,
            agg.sum_bwt_exact_w,
            agg.sum_bwt_sample,
            100.0 * agg.sum_bwt_exact_w as f64 / agg.sum_bwt_sample.max(1) as f64,
            agg.sum_bwt_exact_delta,
            agg.sum_bwt_sample,
            100.0 * agg.sum_bwt_exact_delta as f64 / agg.sum_bwt_sample.max(1) as f64,
            m_del
        );
    }
}

fn main() {
    let t0 = Instant::now();
    let bz_path = "/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2";
    let data = fs::read(bz_path).expect("read wikidata.bz2");
    let kernels = Kernels::detect();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut runs = Runs::default();
    let mut verifier = OpenCells::default();
    let mut ref_out = Vec::new();

    let mut by_stream: Vec<Vec<BlockRec>> = Vec::new();
    let mut total_blocks = 0usize;
    let mut large_blocks = 0usize;
    let mut small_blocks = 0usize;
    let mut admitted_large = 0usize;
    let mut verified_exact = 0usize;

    loop {
        match scanner.next(syms).expect("scan wikidata.bz2") {
            Event::Done => break,
            Event::Block {
                index,
                stream,
                symbols,
                ..
            } => {
                total_blocks += 1;
                let sidx = stream as usize;
                while by_stream.len() <= sidx {
                    by_stream.push(Vec::new());
                }
                let takes = runs::takes_runs(&symbols);
                if takes {
                    let n = runs::front(&symbols, &mut runs, kernels).expect("runs::front");
                    if n >= 65_536 {
                        large_blocks += 1;
                    } else {
                        small_blocks += 1;
                    }
                    let diag = run_open_diag(runs.records(), n, symbols.orig_ptr, LIMIT);
                    // Cross-check exactness against production OpenCells on every block!
                    let ok = verifier.invert_runs(&runs, n, symbols.orig_ptr, &mut ref_out, kernels);
                    assert!(ok && ref_out == diag.out, "OpenCells mismatch on block {}", index);
                    verified_exact += 1;

                    let admitted = check_admitted(runs.records().len(), n, diag.m, diag.strips.len());
                    if n >= 65_536 && admitted {
                        admitted_large += 1;
                    }
                    let toks = classify_tokens(&diag.out);
                    let in_stream = by_stream[sidx].len();
                    by_stream[sidx].push(BlockRec {
                        stream: sidx,
                        in_stream,
                        global_idx: index as usize,
                        diag,
                        toks,
                        admitted,
                    });
                } else {
                    // Track non-runs block (likely tiny or high-entropy)
                    small_blocks += 1;
                }
                syms = symbols.syms;
            }
        }
    }

    println!(
        "Scanned {} blocks across {} windows in {:.2}s: large(>=64K)={} (admitted={}), small/non-runs={}, exact-verified against mibz2::OpenCells={}",
        total_blocks,
        by_stream.len(),
        t0.elapsed().as_secs_f64(),
        large_blocks,
        admitted_large,
        small_blocks,
        verified_exact
    );

    // Check block size distribution per window!
    for (w, blks) in by_stream.iter().enumerate() {
        let lg = blks.iter().filter(|b| b.diag.n >= 65_536).count();
        let adm = blks.iter().filter(|b| b.diag.n >= 65_536 && b.admitted).count();
        let sm = blks.len() - lg;
        let avg_m = if adm > 0 {
            blks.iter()
                .filter(|b| b.diag.n >= 65_536 && b.admitted)
                .map(|b| 100.0 * b.diag.m as f64 / b.diag.n as f64)
                .sum::<f64>()
                / adm as f64
        } else {
            0.0
        };
        println!(
            "  w{:02}: total_run_blocks={:2}, large={:2} (admitted={:2}, avg m16/n={:5.2}%), small={:2}",
            w, blks.len(), lg, adm, avg_m, sm
        );
    }

    let mut small_ns: Vec<usize> = by_stream
        .iter()
        .flat_map(|v| v.iter())
        .filter(|b| b.diag.n < 65_536)
        .map(|b| b.diag.n)
        .collect();
    small_ns.sort_unstable();
    if !small_ns.is_empty() {
        println!(
            "  Small run-route blocks ({}): min={}B, med={}B, max={}B, total={}B",
            small_ns.len(),
            small_ns[0],
            small_ns[small_ns.len() / 2],
            small_ns[small_ns.len() - 1],
            small_ns.iter().sum::<usize>()
        );
    }

    // 1. Genuinely consecutive large admitted pairs (B_k, B_{k+1}) within each window
    let mut agg_consec_all = PairAgg::default();
    let mut agg_consec_strict = PairAgg::default(); // strictly adjacent in stream (in_stream+1) AND both large admitted
    let mut agg_stride4 = PairAgg::default(); // worker-local at 4 workers: large[i] -> large[i+4]

    for blks in &by_stream {
        // Strictly adjacent in stream (global_idx + 1) where both are large admitted
        for pair in blks.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            if b.global_idx == a.global_idx + 1
                && a.diag.n >= 65_536
                && b.diag.n >= 65_536
                && a.admitted
                && b.admitted
            {
                eval_pair(a, b, &mut agg_consec_strict, true, true);
            }
        }
        // Consecutive large admitted blocks in same window (skipping tiny 14-byte trailer/divider blocks if any)
        let large_adm: Vec<&BlockRec> = blks
            .iter()
            .filter(|b| b.diag.n >= 65_536 && b.admitted)
            .collect();
        for pair in large_adm.windows(2) {
            eval_pair(pair[0], pair[1], &mut agg_consec_all, true, true);
        }
        for i in 0..large_adm.len().saturating_sub(4) {
            eval_pair(large_adm[i], large_adm[i + 4], &mut agg_stride4, false, false);
        }
    }

    print_agg(
        "STRICTLY ADJACENT LARGE ADMITTED PAIRS (global_idx+1, same window)",
        &mut agg_consec_strict,
        true,
        true,
    );
    print_agg(
        "CONSECUTIVE LARGE ADMITTED PAIRS WITHIN SAME WINDOW (skipping tiny blocks)",
        &mut agg_consec_all,
        true,
        true,
    );
    print_agg(
        "4-WORKER STRIDE LARGE ADMITTED PAIRS (large[i] -> large[i+4], same window)",
        &mut agg_stride4,
        false,
        false,
    );

    // 2. Cumulative window history: union of ALL previous large admitted blocks in same window -> B_k
    let mut cum_words_b = 0usize;
    let mut cum_words_sh = 0usize;
    let mut cum_spell_b = 0usize;
    let mut cum_spell_sh = 0usize;
    let mut cum_m16 = 0usize;
    let mut cum_nonlit_sh = 0usize;
    let mut cum_bytes_n = 0usize;
    let mut cum_bytes_sh = 0usize;
    let mut cum_sha1_ret_tot = 0usize;
    let mut cum_sha1_ret_sh = 0usize;
    let mut cum_ent_ret_tot = 0usize;
    let mut cum_ent_ret_sh = 0usize;
    let mut cum_guid_ret_tot = 0usize;
    let mut cum_guid_ret_sh = 0usize;
    let mut cum_key_ret_tot = 0usize;
    let mut cum_key_ret_sh = 0usize;
    let mut cum_blocks = 0usize;

    for blks in &by_stream {
        let mut hist_words: HashSet<&[u8]> = HashSet::new();
        let mut hist_sha1: HashSet<&[u8]> = HashSet::new();
        let mut hist_ent: HashSet<&[u8]> = HashSet::new();
        let mut hist_guid: HashSet<&[u8]> = HashSet::new();
        let mut hist_key: HashSet<&[u8]> = HashSet::new();
        let mut seen_any = false;
        for b in blks.iter().filter(|b| b.diag.n >= 65_536 && b.admitted) {
            if seen_any {
                cum_blocks += 1;
                let wb_dist: HashSet<&[u8]> = b.diag.words.iter().map(|v| v.as_slice()).collect();
                cum_words_b += wb_dist.len();
                cum_words_sh += wb_dist.iter().filter(|w| hist_words.contains(**w)).count();
                cum_spell_b += b.diag.spelling_bytes;
                cum_spell_sh += b
                    .diag
                    .words
                    .iter()
                    .filter(|w| hist_words.contains(w.as_slice()))
                    .map(|w| w.len())
                    .sum::<usize>();
                cum_m16 += b.diag.m;
                cum_bytes_n += b.diag.n;
                for (w, &cnt) in &b.diag.word_emit_counts {
                    if w.len() >= 2 && hist_words.contains(w.as_slice()) {
                        cum_nonlit_sh += cnt;
                        cum_bytes_sh += w.len() * cnt;
                    }
                }
                for &epos in &b.diag.ret_end_tpos {
                    let cls = b.toks.pos_cls[epos];
                    let (s, e) = b.toks.pos_tok_range[epos];
                    let slice = &b.diag.out[s as usize..e as usize];
                    match cls {
                        1 => {
                            cum_guid_ret_tot += 1;
                            if hist_guid.contains(slice) {
                                cum_guid_ret_sh += 1;
                            }
                        }
                        2 => {
                            cum_sha1_ret_tot += 1;
                            if hist_sha1.contains(slice) {
                                cum_sha1_ret_sh += 1;
                            }
                        }
                        3 => {
                            cum_ent_ret_tot += 1;
                            if hist_ent.contains(slice) {
                                cum_ent_ret_sh += 1;
                            }
                        }
                        4 => {
                            cum_key_ret_tot += 1;
                            if hist_key.contains(slice) {
                                cum_key_ret_sh += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            for w in &b.diag.words {
                hist_words.insert(w.as_slice());
            }
            for t in b.toks.sha1s.keys() {
                hist_sha1.insert(t.as_slice());
            }
            for t in b.toks.entities.keys() {
                hist_ent.insert(t.as_slice());
            }
            for t in &b.toks.guids {
                hist_guid.insert(t.as_slice());
            }
            for t in &b.toks.keys {
                hist_key.insert(t.as_slice());
            }
            seen_any = true;
        }
    }

    println!(
        "\n=== CUMULATIVE SAME-WINDOW HISTORY (all prior large admitted blocks in window -> B, {} blocks) ===",
        cum_blocks
    );
    println!(
        "  Open distinct words in history: {}/{} ({:.2}%)",
        cum_words_sh,
        cum_words_b,
        100.0 * cum_words_sh as f64 / cum_words_b.max(1) as f64
    );
    println!(
        "  Open spelling bytes in history: {}/{} B ({:.2}%)",
        cum_spell_sh,
        cum_spell_b,
        100.0 * cum_spell_sh as f64 / cum_spell_b.max(1) as f64
    );
    println!(
        "  Open non-lit rows in history:   {}/{} m16 ({:.2}%), emitted bytes={:.2}% of n",
        cum_nonlit_sh,
        cum_m16,
        100.0 * cum_nonlit_sh as f64 / cum_m16.max(1) as f64,
        100.0 * cum_bytes_sh as f64 / cum_bytes_n.max(1) as f64
    );
    println!(
        "  Retained rows in history by token: GUID={}/{} ({:.2}%), sha1={}/{} ({:.2}%), entity={}/{} ({:.2}%), key={}/{} ({:.2}%)",
        cum_guid_ret_sh,
        cum_guid_ret_tot,
        100.0 * cum_guid_ret_sh as f64 / cum_guid_ret_tot.max(1) as f64,
        cum_sha1_ret_sh,
        cum_sha1_ret_tot,
        100.0 * cum_sha1_ret_sh as f64 / cum_sha1_ret_tot.max(1) as f64,
        cum_ent_ret_sh,
        cum_ent_ret_tot,
        100.0 * cum_ent_ret_sh as f64 / cum_ent_ret_tot.max(1) as f64,
        cum_key_ret_sh,
        cum_key_ret_tot,
        100.0 * cum_key_ret_sh as f64 / cum_key_ret_tot.max(1) as f64,
    );

    // 3. Also check within-block vs cross-block-only sha1 repetition on consecutive pairs!
    let mut sha1_uniq_b_in_a = 0usize;
    let mut sha1_uniq_b_tot = 0usize;
    let mut sha1_rep_b_in_a = 0usize;
    let mut sha1_rep_b_tot = 0usize;
    let mut sha1_ret_uniq_b_in_a = 0usize;
    for blks in &by_stream {
        let large_adm: Vec<&BlockRec> = blks
            .iter()
            .filter(|b| b.diag.n >= 65_536 && b.admitted)
            .collect();
        for pair in large_adm.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            for (tok, &cnt) in &b.toks.sha1s {
                let in_a = a.toks.sha1s.contains_key(tok);
                if cnt == 1 {
                    sha1_uniq_b_tot += 1;
                    if in_a {
                        sha1_uniq_b_in_a += 1;
                        sha1_ret_uniq_b_in_a += 40;
                    }
                } else {
                    sha1_rep_b_tot += cnt;
                    if in_a {
                        sha1_rep_b_in_a += cnt;
                    }
                }
            }
        }
    }
    println!(
        "\n=== WITHIN-BLOCK vs CROSS-BLOCK-ONLY SHA1 (consecutive large admitted pairs) ==="
    );
    println!(
        "  sha1 appearing ONCE in B (unique in B): {}/{} ({:.2}%) appeared in A (~{} retained rows / {} total m16 = {:.3}%)",
        sha1_uniq_b_in_a,
        sha1_uniq_b_tot,
        100.0 * sha1_uniq_b_in_a as f64 / sha1_uniq_b_tot.max(1) as f64,
        sha1_ret_uniq_b_in_a,
        agg_consec_all.sum_m16,
        100.0 * sha1_ret_uniq_b_in_a as f64 / agg_consec_all.sum_m16.max(1) as f64
    );
    println!(
        "  sha1 appearing >=2x in B (already repeated within B): {}/{} occurrences ({:.2}%) also in A",
        sha1_rep_b_in_a,
        sha1_rep_b_tot,
        100.0 * sha1_rep_b_in_a as f64 / sha1_rep_b_tot.max(1) as f64
    );
    println!("\nTotal elapsed: {:.2}s", t0.elapsed().as_secs_f64());
}
