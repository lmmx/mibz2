//! The phrase census (docs/journal/2026-11-10-phrase-census.md): how many dependent *events*
//! an inverse BWT needs on a block's actual trajectory when it works on Ψ's translation cells
//! rather than on rows. It counts; it times nothing, and nothing in the decoder calls it.
//!
//! ```text
//! phrase_census FILE.bz2 [EVERY]     one JSON line a full block (stdout), a pooled summary (stderr)
//! ```
//!
//! **The objects.** F convention: the walk's state is y_0 = orig_ptr, y_{t+1} = Ψ(y_t), and it
//! emits F[y_t] (the bytes of `bwt::walk`, checked). A *cell* is a maximal row interval on
//! which Ψ is a translation and F is constant; S is the set of cell starts, |S| = r (the runs of
//! L, plus the rare bucket boundary inside a translation).
//!
//! **Phrases.** Rows q − 1 and q stay adjacent under Ψ^t until Ψ^t(q) ∈ S. So the text is cut
//! into r phrases at the times the walk lands on S, and inside a phrase the walk runs parallel
//! to the trajectory of the row above: T[t] = T[t + Δ] with Δ constant on the phrase.
//!
//! **What is counted:**
//! - `phr_*`: the phrases' lengths on the trajectory (bytes by length class);
//! - `r2..r64`: the cells of Ψ^s, |S ∪ Ψ⁻¹S ∪ … ∪ Ψ^{-(s-1)}S|, and those a stride-s walk visits;
//! - `split_*`: the first-split word walk: from a cell, step while the whole cell stays inside
//!   one cell; one event a word;
//! - `lz_*`: the online copy walk in three source policies (see `lz`): real Ψ steps, copy
//!   events and copied bytes, with every copy checked against the trajectory.

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const NONE: u32 = u32::MAX;
const CLASSES: [usize; 9] = [1, 2, 4, 8, 16, 32, 64, 128, usize::MAX];

fn class(len: usize) -> usize {
    CLASSES.iter().position(|&c| len <= c).unwrap()
}

struct Col {
    n: usize,
    psi: Vec<u32>,
    /// S, as a flag a row
    start: Vec<bool>,
    /// each row's cell
    cell: Vec<u32>,
    /// each cell's [a, b)
    a: Vec<u32>,
    b: Vec<u32>,
    /// the trajectory: traj[t] = y_t
    traj: Vec<u32>,
    runs: usize,
}

fn column(l: &[u8], orig_ptr: u32, cap: usize) -> Col {
    let n = l.len();
    let mut cnt = [0usize; 256];
    for &c in l {
        cnt[c as usize] += 1;
    }
    let mut next = [0usize; 256];
    let mut f = vec![0u8; n];
    let mut cum = 0;
    for c in 0..256 {
        next[c] = cum;
        f[cum..cum + cnt[c]].fill(c as u8);
        cum += cnt[c];
    }
    let mut psi = vec![0u32; n];
    for (i, &c) in l.iter().enumerate() {
        psi[next[c as usize]] = i as u32;
        next[c as usize] += 1;
    }
    let mut start = vec![false; n];
    let mut cell = vec![0u32; n];
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for q in 0..n {
        start[q] = q == 0 || psi[q] != psi[q - 1] + 1 || f[q] != f[q - 1];
        // a capped cell is cut every `cap` rows from its own start
        if !start[q] && cap > 0 && q - *a.last().unwrap() as usize >= cap {
            start[q] = true;
        }
        if start[q] {
            if let Some(last) = b.last_mut() {
                *last = q as u32;
            }
            a.push(q as u32);
            b.push(n as u32);
        }
        cell[q] = (a.len() - 1) as u32;
    }
    let mut traj = Vec::with_capacity(n);
    let mut y = orig_ptr;
    for _ in 0..n {
        traj.push(y);
        y = psi[y as usize];
    }
    let runs = 1 + l.windows(2).filter(|w| w[0] != w[1]).count();
    // the trajectory's bytes are the walk's
    debug_assert!(traj.iter().all(|&y| (y as usize) < n));
    Col {
        n,
        psi,
        start,
        cell,
        a,
        b,
        traj,
        runs,
    }
}

/// The online copy walk. A step at time t in cell k either copies from an earlier visit of the
/// cell (its source), for as long as the two trajectories stay in one cell, or, with no source,
/// takes one real Ψ step.
/// - policy 0: the source is the cell's first visit, registered by real steps only;
/// - policy 1: the nearer in rows of the cell's first visit and its latest event start;
/// - policy 2: the nearest visited row above or below within `reach` rows, every visit registered
///   (an upper bound on what any neighbour-copy can do, at a registration cost of its own).
///
/// Returns (real steps, copy events, bytes by copy-length class, copies by class).
fn lz(c: &Col, policy: u8, reach: usize) -> (usize, usize, [usize; 9], [usize; 9]) {
    let n = c.n;
    let cells = c.a.len();
    let mut first = vec![NONE; cells];
    let mut last = vec![NONE; cells];
    let mut vt = if policy == 2 {
        vec![NONE; n]
    } else {
        Vec::new()
    };
    let (mut real, mut copies) = (0, 0);
    let (mut bytes, mut count) = ([0usize; 9], [0usize; 9]);
    // the steps the trajectories from times q < t stay in one cell. Equal row distance is not
    // enough (the Researcher's `aab`, review 6755014327): two rows of different cells can keep
    // their distance for a step and emit different bytes. Rows of one cell emit one byte and
    // keep their distance for one more step, so the certificate is the cell, step by step.
    let coherent = |q: usize, t: usize| -> usize {
        let mut m = 0;
        while t + m < n && c.cell[c.traj[t + m] as usize] == c.cell[c.traj[q + m] as usize] {
            m += 1;
        }
        m
    };
    let rows_apart = |q: usize, t: usize| (c.traj[t] as i64 - c.traj[q] as i64).unsigned_abs();
    let mut t = 0;
    while t < n {
        let y = c.traj[t] as usize;
        let k = c.cell[y] as usize;
        let mut best = 0;
        match policy {
            0 => {
                if first[k] != NONE {
                    best = coherent(first[k] as usize, t);
                }
            }
            1 => {
                // the nearer in rows, as the prototype chooses: never by the copy's length
                let q = [first[k], last[k]]
                    .into_iter()
                    .filter(|&q| q != NONE)
                    .min_by_key(|&q| rows_apart(q as usize, t));
                if let Some(q) = q {
                    best = coherent(q as usize, t);
                }
            }
            _ => {
                let (a, b) = (c.a[k] as usize, c.b[k] as usize);
                let lo = a.max(y.saturating_sub(reach));
                let hi = b.min(y + 1 + reach);
                let above = (lo..y).rev().find(|&q| vt[q] != NONE);
                let below = (y + 1..hi).find(|&q| vt[q] != NONE);
                // the nearer in rows of the two, the row above on a tie
                let q = match (above, below) {
                    (Some(u), Some(v)) => Some(if y - u <= v - y { u } else { v }),
                    (u, v) => u.or(v),
                };
                if let Some(q) = q {
                    best = coherent(vt[q] as usize, t);
                }
            }
        }
        if first[k] == NONE {
            first[k] = t as u32;
        }
        last[k] = t as u32;
        let m = if best == 0 {
            real += 1;
            1
        } else {
            copies += 1;
            bytes[class(best)] += best;
            count[class(best)] += 1;
            best
        };
        if policy == 2 {
            for i in 0..m {
                vt[c.traj[t + i] as usize] = (t + i) as u32;
            }
        }
        t += m;
    }
    (real, copies, bytes, count)
}

/// A tube packet: from its cell at offsets lo..=hi, the next m bytes are the bytes at time t,
/// and the walk lands at row `end` + (offset − o)
#[derive(Clone, Copy)]
struct Packet {
    lo: u32,
    hi: u32,
    o: u32,
    t: u32,
    m: u32,
    end: u32,
}

struct Open {
    k: usize,
    o: u32,
    t: u32,
    al: u32,
    be: u32,
    m: u32,
}

/// The tube walk: event-level memory only. A step in cell k at offset o either hits one of
/// the cell's packets (copies m bytes, lands by translation) or takes one real Ψ step. Each
/// event's slack narrows the open packets' tubes; a packet closes before an event that would
/// shrink its area (width × bytes), and is stored in its cell. Every hit is checked against
/// the trajectory. Returns (real steps, hits, bytes by hit-length class, hits by class,
/// packets stored, mean landing span in cells).
fn tube(
    c: &Col,
    depth: usize,
    slots: usize,
    open_on_hit: bool,
) -> (usize, usize, [usize; 9], [usize; 9], usize) {
    let n = c.n;
    let cells = c.a.len();
    let mut memo: Vec<Vec<Packet>> = vec![Vec::new(); cells];
    let mut open: Vec<Open> = Vec::new();
    let (mut real, mut hits, mut stored) = (0, 0, 0);
    let (mut bytes, mut count) = ([0usize; 9], [0usize; 9]);
    let mut t = 0usize;
    while t < n {
        let y = c.traj[t];
        let k = c.cell[y as usize] as usize;
        let o = y - c.a[k];
        let len = c.b[k] - c.a[k];
        // the longest packet of the cell whose tube holds this offset
        let hit = memo[k]
            .iter()
            .filter(|p| p.lo <= o && o <= p.hi && t + p.m as usize <= n)
            .max_by_key(|p| p.m)
            .copied();
        let (ev, sl, sh) = match hit {
            Some(p) => {
                // checked: the same displacement through the whole packet, and the landing row
                let d = y as i64 - c.traj[p.t as usize] as i64;
                for i in 0..p.m as usize {
                    assert_eq!(c.traj[t + i] as i64 - c.traj[p.t as usize + i] as i64, d);
                }
                if t + (p.m as usize) < n {
                    assert_eq!(
                        c.traj[t + p.m as usize] as i64,
                        p.end as i64 + o as i64 - p.o as i64
                    );
                }
                hits += 1;
                bytes[class(p.m as usize)] += p.m as usize;
                count[class(p.m as usize)] += 1;
                (p.m, o - p.lo, p.hi - o)
            }
            None => {
                real += 1;
                (1, o, len - 1 - o)
            }
        };
        // narrow the open packets, closing those this event would shrink in area
        let mut i = 0;
        while i < open.len() {
            let q = &mut open[i];
            let (na, nb) = (q.al.min(sl), q.be.min(sh));
            let (old, new) = (
                (q.al + q.be + 1) as u64 * q.m as u64,
                (na + nb + 1) as u64 * (q.m + ev) as u64,
            );
            if new < old || q.m + ev > 4095 {
                let p = Packet {
                    lo: q.o - q.al,
                    hi: q.o + q.be,
                    o: q.o,
                    t: q.t,
                    m: q.m,
                    end: y,
                };
                if p.hi > p.lo {
                    let slot = &mut memo[q.k];
                    if slot.len() < slots {
                        slot.push(p);
                        stored += 1;
                    } else {
                        // evict the smallest area if the new one is larger
                        let area = |p: &Packet| (p.hi - p.lo + 1) as u64 * p.m as u64;
                        let (j, least) = slot
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, p)| area(p))
                            .map(|(j, p)| (j, area(p)))
                            .unwrap();
                        if area(&p) > least {
                            slot[j] = p;
                        }
                    }
                }
                open.remove(i);
            } else {
                q.al = na;
                q.be = nb;
                q.m += ev;
                i += 1;
            }
        }
        // open a packet here: a multi-row cell, wider than every open tube
        if (hit.is_none() || open_on_hit)
            && sl + sh > 0
            && open.len() < depth
            && open.iter().all(|q| q.k != k && q.al + q.be < sl + sh)
        {
            open.push(Open {
                k,
                o,
                t: t as u32,
                al: sl,
                be: sh,
                m: ev,
            });
        }
        t += ev as usize;
    }
    (real, hits, bytes, count, stored)
}

fn census(l: &[u8], orig_ptr: u32, walked: &[u8]) -> String {
    let c = column(l, orig_ptr, 0);
    let n = c.n;
    let cells = c.a.len();
    // the trajectory emits the walk's bytes
    {
        let mut cnt = [0usize; 256];
        for &x in l {
            cnt[x as usize] += 1;
        }
        let mut f = Vec::with_capacity(n);
        for (x, &k) in cnt.iter().enumerate() {
            f.extend(std::iter::repeat(x as u8).take(k));
        }
        assert!(c.traj.iter().zip(walked).all(|(&y, &w)| f[y as usize] == w));
    }

    // phrases: the walk's landings on S
    let (mut phr_bytes, mut phr_count) = ([0usize; 9], [0usize; 9]);
    let landings: Vec<usize> = (0..n).filter(|&t| c.start[c.traj[t] as usize]).collect();
    assert_eq!(landings.len(), cells);
    let mut sq = 0f64;
    for (i, &t) in landings.iter().enumerate() {
        let e = if i + 1 < landings.len() {
            landings[i + 1]
        } else {
            n + landings[0]
        };
        let len = e - t;
        phr_bytes[class(len)] += len;
        phr_count[class(len)] += 1;
        sq += (len * len) as f64;
    }

    // K[q]: the steps rows q − 1 and q stay adjacent; and each cell's first-split word
    let mut kk = vec![0u32; n];
    {
        // in reverse text order from a landing: K at a start is 0, else 1 + K at the next row
        let t0 = landings[0];
        let mut next = 0u32;
        for i in (0..n).rev() {
            let t = (t0 + i) % n;
            let y = c.traj[t] as usize;
            let k = if c.start[y] { 0 } else { next + 1 };
            kk[y] = k;
            next = k;
        }
    }
    // a cell [a, b) stays whole for w = min K[q], q in (a, b), steps: rows q - 1 and q are in
    // one cell at steps 0 .. K[q] - 1, and at step K[q] row q's image is a start, so the two
    // images lie in different cells and may emit different bytes. (Until 2026-11-10 this was
    // 1 + min K: one byte too many. The Optimiser's witness: "aab", cell [0, 2), K[1] = 1,
    // rows emit "aa" and "ab". Corrected; `check_word` below now asserts every word and exit.)
    let cap = 64u32;
    let word: Vec<u32> = (0..cells)
        .map(|k| {
            let (a, b) = (c.a[k] as usize, c.b[k] as usize);
            (a + 1..b).map(|q| kk[q]).min().map_or(cap, |m| m.min(cap))
        })
        .collect();
    // pos[y]: the time the trajectory is at row y
    let mut pos = vec![0u32; n];
    for (t, &y) in c.traj.iter().enumerate() {
        pos[y as usize] = t as u32;
    }
    // the certificate of one word step: from row traj[t] in cell k, the next w bytes are the
    // bytes the cell's first row emits, and the row after them is the first row's image plus
    // this row's offset
    let check_word = |t: usize, k: usize, w: usize| {
        let a = c.a[k] as usize;
        let y = c.traj[t] as usize;
        let ta = pos[a] as usize;
        for j in 0..w {
            assert!(
                c.cell[c.traj[(ta + j) % n] as usize] == c.cell[c.traj[(t + j) % n] as usize],
                "a word step leaves its head's cells at byte {j}"
            );
        }
        if t + w < n {
            assert_eq!(
                c.traj[(t + w) % n] as usize - c.traj[(ta + w) % n] as usize,
                y - a,
                "a word step's exit is not its head's exit plus the offset"
            );
        }
    };
    let (mut split_events, mut split_single, mut split_first, mut split_first_bytes) = (0, 0, 0, 0);
    let (mut split_bytes, mut split_count) = ([0usize; 9], [0usize; 9]);
    {
        let mut seen = vec![false; cells];
        let mut t = 0;
        while t < n {
            let k = c.cell[c.traj[t] as usize] as usize;
            let single = c.b[k] - c.a[k] == 1;
            // a one-row cell has no neighbour to reuse: it steps once
            let w = if single { 1 } else { word[k] as usize }.min(n - t);
            check_word(t, k, w);
            split_events += 1;
            split_single += single as usize;
            if !seen[k] && !single {
                seen[k] = true;
                split_first += 1;
                split_first_bytes += w;
            }
            split_bytes[class(w)] += w;
            split_count[class(w)] += 1;
            t += w;
        }
    }

    // TUNNEL set: the whole-cell words at several caps (docs/journal/2026-11-10-tunnels.md), and
    // nothing else. A cell of w > 1 rows whose rows stay adjacent for h steps is a tunnel of
    // width w and height h; its word is those h bytes, stored once.
    if std::env::var_os("TUNNEL").is_some() {
        let multi: Vec<usize> = (0..cells).filter(|&k| c.b[k] - c.a[k] > 1).collect();
        // the uncapped height of each multi-row cell
        let raw: Vec<usize> = (0..cells)
            .map(|k| {
                let (a, b) = (c.a[k] as usize, c.b[k] as usize);
                (a + 1..b).map(|q| kk[q] as usize).min().unwrap_or(1)
            })
            .collect();
        let rows_multi: usize = multi.iter().map(|&k| (c.b[k] - c.a[k]) as usize).sum();
        // (w - 1)(h - 1) summed: the rows a tunnelling of every cell's block would remove,
        // were the blocks disjoint (they are not: an upper bound on what collapses)
        let area: usize = multi
            .iter()
            .map(|&k| ((c.b[k] - c.a[k]) as usize - 1) * (raw[k].min(n) - 1))
            .sum();
        let mut out = format!(
            "{{\"n\":{n},\"cells\":{cells},\"multi\":{},\"rows_multi\":{rows_multi},\"area\":{area},\"tun\":[",
            multi.len()
        );
        for (i, cap) in [2usize, 4, 8, 16, 32, 64, 128, 256, usize::MAX]
            .into_iter()
            .enumerate()
        {
            let mut seen = vec![false; cells];
            let (mut events, mut used, mut used_bytes, mut single_events) = (0usize, 0, 0, 0);
            let (mut short_events, mut short_bytes) = (0usize, 0usize);
            let (mut tall_events, mut tall_bytes) = (0usize, 0usize);
            let mut t = 0;
            while t < n {
                let k = c.cell[c.traj[t] as usize] as usize;
                let single = c.b[k] - c.a[k] == 1;
                let full = if single { 1 } else { raw[k].min(cap) };
                let w = full.min(n - t);
                check_word(t, k, w);
                events += 1;
                single_events += single as usize;
                if w > 32 {
                    tall_events += 1;
                    tall_bytes += w;
                }
                if w <= 2 {
                    short_events += 1;
                    short_bytes += w;
                }
                if !single && !seen[k] {
                    seen[k] = true;
                    used += 1;
                    used_bytes += full;
                }
                t += w;
            }
            // every multi-row cell's word, were all of them built
            let all_bytes: usize = multi.iter().map(|&k| raw[k].min(cap).min(n)).sum();
            out += &format!(
                "{}[{},{events},{single_events},{used},{used_bytes},{all_bytes},{short_events},{short_bytes},{tall_events},{tall_bytes}]",
                if i == 0 { "" } else { "," },
                if cap == usize::MAX { 0 } else { cap }
            );
        }
        out += "],\"hybrid\":[";
        // the stride-k walk that copies tall cells: at a landing in a Ψ^k cell (rows with
        // K >= k between them) whose rows stay adjacent 33 steps or more, take the whole word
        // (to the cap); else take k bytes. Every tall step is checked against the cell's head.
        let mut first = true;
        for k in [4usize, 8] {
            // head[y]: the first row of y's Ψ^k cell; hk[head]: min K over its interior
            let mut head = vec![0u32; n];
            let mut hk = vec![0u32; n];
            let mut a = 0usize;
            let mut m = u32::MAX;
            for q in 0..n {
                if q > 0 && (kk[q] as usize) < k {
                    hk[a] = m;
                    a = q;
                    m = u32::MAX;
                } else if q > 0 {
                    m = m.min(kk[q]);
                }
                head[q] = a as u32;
            }
            hk[a] = m;
            for cap in [64usize, 256] {
                let (mut steps, mut tall_steps, mut tall_bytes) = (0usize, 0usize, 0usize);
                let mut t = 0;
                while t < n {
                    let y = c.traj[t] as usize;
                    let a = head[y] as usize;
                    let h = hk[a];
                    let w = if h != u32::MAX && h as usize >= 33 {
                        let w = (h as usize).min(cap).min(n - t);
                        // the certificate, against the Ψ^k cell's head
                        let ta = pos[a] as usize;
                        for j in 0..w {
                            assert!(
                                c.cell[c.traj[(ta + j) % n] as usize]
                                    == c.cell[c.traj[(t + j) % n] as usize],
                                "a tall step leaves its head's cells at byte {j}"
                            );
                        }
                        if t + w < n {
                            assert_eq!(
                                c.traj[(t + w) % n] as usize - c.traj[(ta + w) % n] as usize,
                                y - a
                            );
                        }
                        tall_steps += 1;
                        tall_bytes += w;
                        w
                    } else {
                        k.min(n - t)
                    };
                    steps += 1;
                    t += w;
                }
                // the tall cells and their words, were all of them stored
                let (mut tall_cells, mut tall_words) = (0usize, 0usize);
                for q in 0..n {
                    if head[q] as usize == q && hk[q] != u32::MAX && hk[q] as usize >= 33 {
                        tall_cells += 1;
                        tall_words += (hk[q] as usize).min(cap);
                    }
                }
                out += &format!(
                    "{}[{k},{cap},{steps},{tall_steps},{tall_bytes},{tall_cells},{tall_words}]",
                    if first { "" } else { "," }
                );
                first = false;
            }
        }
        return out + "]}";
    }

    // the cells of Ψ^s, and those a stride-s walk from the origin lands in
    let mut rs = Vec::new();
    {
        let mut cur = c.start.clone();
        let mut s = 1;
        for target in [2usize, 4, 8, 16, 32, 64] {
            while s < target {
                // cut_{s+1}[q] = S[q] or cut_s[Ψ(q)]
                let nxt: Vec<bool> = (0..n)
                    .map(|q| c.start[q] || cur[c.psi[q] as usize])
                    .collect();
                cur = nxt;
                s += 1;
            }
            let total = cur.iter().filter(|&&x| x).count();
            // the cells of Ψ^s a stride-s walk visits: distinct cell ids at times 0, s, 2s, …
            let mut id = vec![0u32; n];
            let mut k = 0u32;
            for q in 0..n {
                if cur[q] && q > 0 {
                    k += 1;
                }
                id[q] = k;
            }
            let mut seen = vec![false; k as usize + 1];
            let mut visited = 0;
            let mut t = 0;
            while t < n {
                let i = id[c.traj[t] as usize] as usize;
                if !seen[i] {
                    seen[i] = true;
                    visited += 1;
                }
                t += target;
            }
            rs.push((target, total, visited));
        }
    }

    let mut out = format!(
        "{{\"n\":{n},\"runs\":{},\"cells\":{cells},\"phr_sizebiased\":{:.2},\"phr_bytes\":{:?},\"phr_count\":{:?}",
        c.runs,
        sq / n as f64,
        phr_bytes,
        phr_count
    );
    for (s, total, visited) in rs {
        out += &format!(",\"r{s}\":{total},\"v{s}\":{visited}");
    }
    out += &format!(
        ",\"split_events\":{split_events},\"split_single\":{split_single},\"split_first\":{split_first},\"split_first_bytes\":{split_first_bytes},\"split_bytes\":{split_bytes:?},\"split_count\":{split_count:?}"
    );
    for (name, policy, reach) in [("lz0", 0u8, 0usize), ("lz1", 1, 0), ("lz2", 2, 64)] {
        let (real, copies, bytes, count) = lz(&c, policy, reach);
        assert_eq!(real + bytes.iter().sum::<usize>(), n);
        out += &format!(
            ",\"{name}_real\":{real},\"{name}_copies\":{copies},\"{name}_bytes\":{bytes:?},\"{name}_count\":{count:?}"
        );
    }
    let c63 = column(l, orig_ptr, 63);
    out += &format!(",\"cells63\":{}", c63.a.len());
    for (name, depth, slots, on_hit) in [
        ("tube_d1s1", 1usize, 1usize, false),
        ("tube_d4s1", 4, 1, false),
        ("tube_d4s2", 4, 2, false),
        ("tube_d4s2h", 4, 2, true),
        ("tube_d8s4h", 8, 4, true),
    ] {
        let (real, hits, bytes, count, stored) = tube(&c63, depth, slots, on_hit);
        assert_eq!(real + bytes.iter().sum::<usize>(), n);
        out += &format!(
            ",\"{name}_real\":{real},\"{name}_copies\":{hits},\"{name}_bytes\":{bytes:?},\"{name}_count\":{count:?},\"{name}_stored\":{stored}"
        );
    }
    out + "}"
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1]).expect("read");
    let every: usize = args.get(2).map_or(1, |s| s.parse().unwrap());
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut ll, mut counts, mut tt) = (Vec::new(), [0u32; 256], TVector::new());
    let mut syms = Vec::new();
    let mut walked = Vec::new();
    let mut i = 0;
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        if ll.len() >= 65536 {
            if i % every == 0 {
                build(&ll, &counts, &mut tt);
                walked.clear();
                walk(&tt, b.orig_ptr, &mut walked);
                println!("{}", census(&ll, b.orig_ptr, &walked));
            }
            i += 1;
        }
        syms = b.syms;
    }
}
