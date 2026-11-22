use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct BlockMeta {
    file: &'static str,
    n: usize,
    p: usize,
    r: usize,
}

const BLOCKS: &[BlockMeta] = &[
    BlockMeta { file: "block-0.L",   n: 899931, p: 264045, r: 51184 },
    BlockMeta { file: "block-2.L",   n: 900000, p: 320807, r: 54532 },
    BlockMeta { file: "block-85.L",  n: 899935, p: 325088, r: 79375 },
    BlockMeta { file: "block-143.L", n: 899869, p: 802868, r: 108335 },
    BlockMeta { file: "block-207.L", n: 899998, p: 205180, r: 87111 },
    BlockMeta { file: "block-210.L", n: 899959, p: 791429, r: 128732 },
];

/// Plain reference inverse BWT (works for ANY column L, including periodic & multi-cycle).
/// Starts at row p, follows Psi = LF^{-1}, emits F[x] until returning to p or n bytes,
/// then repeats cycle to n bytes (exact bzip2/Open semantics).
fn plain_inverse(l: &[u8], p: usize) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut first = [0u32; 256];
    let mut sum = 0u32;
    for c in 0..256 {
        first[c] = sum;
        sum += counts[c];
    }
    let mut cur = first;
    let mut psi = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let j = cur[b as usize];
        cur[b as usize] += 1;
        psi[j as usize] = i as u32;
    }
    let mut f_col = vec![0u8; n];
    for c in 0..256 {
        let s = first[c] as usize;
        let e = if c == 255 { n } else { first[c + 1] as usize };
        for x in s..e {
            f_col[x] = c as u8;
        }
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p;
    for _ in 0..n {
        out.push(f_col[x]);
        x = psi[x] as usize;
        if x == p {
            break;
        }
    }
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
    out
}

/// Build Open's retained map (edges, words, mark, bits) for a given limit.
struct OpenMap {
    m: usize,
    mark: u32,
    bits: u32,
    edges: Vec<u32>,
    words: Vec<Vec<u8>>,
}

fn build_open_map(l: &[u8], p: usize, limit: usize) -> OpenMap {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut first = [0u32; 256];
    let mut sum = 0u32;
    for c in 0..256 {
        first[c] = sum;
        sum += counts[c];
    }
    let mut cur = first;
    let mut psi = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let j = cur[b as usize];
        cur[b as usize] += 1;
        psi[j as usize] = i as u32;
    }
    let mut f_col = vec![0u8; n];
    for c in 0..256 {
        let s = first[c] as usize;
        let e = if c == 255 { n } else { first[c + 1] as usize };
        for x in s..e {
            f_col[x] = c as u8;
        }
    }

    // Maximal runs & split
    let mut is_ret = vec![false; n];
    let mut i = 0;
    while i < n {
        let c = l[i];
        let start = i;
        i += 1;
        while i < n && l[i] == c {
            i += 1;
        }
        let w = i - start;
        if w <= limit {
            for x in start..i {
                is_ret[x] = true;
            }
        } else if p >= start && p < i {
            is_ret[p] = true;
        }
    }
    let mut ret_rows = Vec::new();
    let mut row_to_rank = vec![u32::MAX; n];
    for x in 0..n {
        if is_ret[x] {
            row_to_rank[x] = ret_rows.len() as u32;
            ret_rows.push(x as u32);
        }
    }
    let m = ret_rows.len();
    let mark = row_to_rank[p];
    // Choose bits so m < (1 << bits) (strictly!), ensuring next + 1 NEVER carries into code bits!
    let bits = (u32::BITS - (m as u32).leading_zeros()).max(1);
    assert!(m < (1usize << bits));

    let mut words: Vec<Vec<u8>> = (0..=255u8).map(|b| vec![b]).collect();
    let mut word_map: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut edges = vec![0u32; m];
    let mut buf = Vec::new();

    for (u, &row) in ret_rows.iter().enumerate() {
        buf.clear();
        let mut x = row as usize;
        loop {
            buf.push(f_col[x]);
            x = psi[x] as usize;
            if is_ret[x] {
                break;
            }
        }
        let next = row_to_rank[x];
        let code = if buf.len() == 1 {
            buf[0] as u32
        } else if let Some(&id) = word_map.get(&buf) {
            id
        } else {
            let id = words.len() as u32;
            words.push(buf.clone());
            word_map.insert(buf.clone(), id);
            id
        };
        edges[u] = (code << bits) | next;
    }

    OpenMap {
        m,
        mark,
        bits,
        edges,
        words,
    }
}

#[derive(Default, Debug)]
struct LazyStats {
    walk_steps: usize,         // total dependent edge loads in emit
    tunnel_hits: usize,        // steps that hit a recorded tunnel shortcut
    tunnels_recorded: usize,   // distinct tunnel shortcuts recorded
    shortcut_writes: usize,    // total edges[] entries overwritten with shortcuts
    bytes_by_tunnel: usize,    // output bytes copied via tunnel shortcuts
    neighbor_checks: usize,    // total adjacent edge comparisons done
}

/// Exact Lazy Output-Backed Tunnel Memoization on Open's `edges: Vec<u32>`.
/// Theorem: never includes `mark` in any strip of width >= 2, so no shortcut ever skips `mark`.
fn emit_lazy_tunnel(
    om: &OpenMap,
    n: usize,
    min_steps: usize,
    max_slack: usize,
    record_sub_tunnels: bool,
) -> (Vec<u8>, LazyStats) {
    let m = om.m;
    let mark = om.mark as usize;
    let bits = om.bits;
    let mask = (1u32 << bits) - 1;
    let tunnel_base = om.words.len() as u32;
    // Check if tunnel codes fit in u32 (if not, cap number of tunnels)
    let max_codes = (u32::MAX >> bits) as usize;

    let mut edges = om.edges.clone();
    // Tunnel table: (out_pos, byte_len)
    let mut tunnels: Vec<(u32, u32)> = Vec::new();
    let mut stats = LazyStats::default();
    let mut out = Vec::with_capacity(n);

    // Active tunnel tracking state:
    // We track a stack of (or single) live tunnel start(s).
    // Even simpler: maintain the current active tunnel `(start_u, start_out, left, right, steps)`.
    // If `record_sub_tunnels` is true, when width narrows from w_old to w_new >= 2, we can record the
    // wider outer tunnel (if steps >= min_steps) before continuing with the narrower inner tunnel!
    let mut tracking = false;
    let mut t_start_u: usize = 0;
    let mut t_start_out: usize = 0;
    let mut t_left: usize = 0;
    let mut t_right: usize = 0;
    let mut t_steps: usize = 0;

    // Helper: compute (left, right) slack of `edges[u]` (excluding `mark` and tunnel codes!)
    let get_slack = |edges: &[u32], u: usize, e: u32, max_s: usize, checks: &mut usize| -> (usize, usize) {
        if u == mark {
            return (0, 0);
        }
        let code = e >> bits;
        if code >= tunnel_base {
            return (0, 0);
        }
        let mut l = 0usize;
        while l < max_s && u > l {
            let idx = u - l - 1;
            if idx == mark {
                break;
            }
            *checks += 1;
            if edges[idx] == e.wrapping_sub((l + 1) as u32) {
                l += 1;
            } else {
                break;
            }
        }
        let mut r = 0usize;
        while r < max_s && u + r + 1 < m {
            let idx = u + r + 1;
            if idx == mark {
                break;
            }
            *checks += 1;
            if edges[idx] == e.wrapping_add((r + 1) as u32) {
                r += 1;
            } else {
                break;
            }
        }
        (l, r)
    };

    let record = |edges: &mut [u32],
                  tunnels: &mut Vec<(u32, u32)>,
                  stats: &mut LazyStats,
                  start_u: usize,
                  start_out: usize,
                  end_u: usize,
                  end_out: usize,
                  left: usize,
                  right: usize,
                  steps: usize| {
        if steps < min_steps || left + right == 0 {
            return;
        }
        let tid_code = tunnel_base as usize + tunnels.len();
        if tid_code > max_codes {
            return;
        }
        let byte_len = (end_out - start_out) as u32;
        if byte_len == 0 {
            return;
        }
        tunnels.push((start_out as u32, byte_len));
        stats.tunnels_recorded += 1;
        let code_shifted = (tid_code as u32) << bits;
        // Write shortcut to all rows in [start_u - left ..= start_u + right], EXCEPT start_u itself (already visited)
        // Wait: could start_u be visited again? No, before returning to mark each row in the cycle is visited once!
        // Still, writing only d != 0 avoids 1 useless write; and we must ensure end_u + d is valid (always < m by slack proof!).
        for d in 1..=left {
            let src = start_u - d;
            let dst = end_u - d;
            // Only overwrite normal (non-tunnel) edges so we don't clobber a longer existing shortcut
            if (edges[src] >> bits) < tunnel_base {
                edges[src] = code_shifted | (dst as u32);
                stats.shortcut_writes += 1;
            }
        }
        for d in 1..=right {
            let src = start_u + d;
            let dst = end_u + d;
            if (edges[src] >> bits) < tunnel_base {
                edges[src] = code_shifted | (dst as u32);
                stats.shortcut_writes += 1;
            }
        }
    };

    let mut j = mark;
    for _ in 0..m {
        let u = j;
        let e = edges[u];
        stats.walk_steps += 1;
        let code = e >> bits;
        let next = (e & mask) as usize;

        if code >= tunnel_base {
            // Close any active tunnel BEFORE taking shortcut!
            if tracking {
                record(
                    &mut edges,
                    &mut tunnels,
                    &mut stats,
                    t_start_u,
                    t_start_out,
                    u,
                    out.len(),
                    t_left,
                    t_right,
                    t_steps,
                );
                tracking = false;
            }
            let tid = (code - tunnel_base) as usize;
            let (out_pos, byte_len) = tunnels[tid];
            let take = (byte_len as usize).min(n - out.len());
            let start = out_pos as usize;
            out.extend_from_within(start..start + take);
            stats.tunnel_hits += 1;
            stats.bytes_by_tunnel += take;
            j = next;
        } else {
            // Normal step at row u!
            // Check slack of u (fast path: only check u-1 and u+1 first!)
            let (l, r) = if !tracking {
                if u == mark {
                    (0, 0)
                } else {
                    // Quick 2-neighbor check!
                    let has_l = u > 0 && (u - 1) != mark && {
                        stats.neighbor_checks += 1;
                        edges[u - 1] == e.wrapping_sub(1)
                    };
                    let has_r = u + 1 < m && (u + 1) != mark && {
                        stats.neighbor_checks += 1;
                        edges[u + 1] == e.wrapping_add(1)
                    };
                    if !has_l && !has_r {
                        (0, 0)
                    } else {
                        get_slack(&edges, u, e, max_slack, &mut stats.neighbor_checks)
                    }
                }
            } else {
                // While tracking, we only need to check up to current t_left and t_right!
                let mut l = 0usize;
                while l < t_left && u > l {
                    let idx = u - l - 1;
                    if idx == mark { break; }
                    stats.neighbor_checks += 1;
                    if edges[idx] == e.wrapping_sub((l + 1) as u32) {
                        l += 1;
                    } else {
                        break;
                    }
                }
                let mut r = 0usize;
                while r < t_right && u + r + 1 < m {
                    let idx = u + r + 1;
                    if idx == mark { break; }
                    stats.neighbor_checks += 1;
                    if edges[idx] == e.wrapping_add((r + 1) as u32) {
                        r += 1;
                    } else {
                        break;
                    }
                }
                (l, r)
            };

            if !tracking {
                if l + r > 0 {
                    tracking = true;
                    t_start_u = u;
                    t_start_out = out.len();
                    t_left = l;
                    t_right = r;
                    t_steps = 1;
                }
            } else {
                if l + r == 0 {
                    // Tunnel ended at u (u is the exit row of the tunnel!)
                    record(
                        &mut edges,
                        &mut tunnels,
                        &mut stats,
                        t_start_u,
                        t_start_out,
                        u,
                        out.len(),
                        t_left,
                        t_right,
                        t_steps,
                    );
                    tracking = false;
                } else {
                    if record_sub_tunnels && (l < t_left || r < t_right) && t_steps >= min_steps {
                        // Some outer offsets peeled off at u! Record the peeled-off offsets (or whole outer)
                        // Note: record only overwrites non-tunnel edges, and we can record the peeled offsets specifically:
                        let tid_code = tunnel_base as usize + tunnels.len();
                        if tid_code <= max_codes && out.len() > t_start_out {
                            tunnels.push((t_start_out as u32, (out.len() - t_start_out) as u32));
                            stats.tunnels_recorded += 1;
                            let code_shifted = (tid_code as u32) << bits;
                            for d in (l + 1)..=t_left {
                                let src = t_start_u - d;
                                let dst = u - d;
                                if (edges[src] >> bits) < tunnel_base {
                                    edges[src] = code_shifted | (dst as u32);
                                    stats.shortcut_writes += 1;
                                }
                            }
                            for d in (r + 1)..=t_right {
                                let src = t_start_u + d;
                                let dst = u + d;
                                if (edges[src] >> bits) < tunnel_base {
                                    edges[src] = code_shifted | (dst as u32);
                                    stats.shortcut_writes += 1;
                                }
                            }
                        }
                    }
                    t_left = l;
                    t_right = r;
                    t_steps += 1;
                }
            }

            // Emit normal word
            let word = &om.words[code as usize];
            let take = word.len().min(n - out.len());
            out.extend_from_slice(&word[..take]);
            j = next;
        }

        if j == mark || out.len() >= n {
            break;
        }
    }

    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
    (out, stats)
}

/// Classical Baier BWT Tunneling (Wheeler graph collapsing on raw L with din/dout + stack)
/// Exact verifier on small columns (verifies both exactness AND counts walk steps!).
fn baier_tbwt_invert_exact(l: &[u8], p: usize) -> (Vec<u8>, usize, usize) {
    let n = l.len();
    // Build LF and F
    let mut counts = [0u32; 256];
    for &b in l { counts[b as usize] += 1; }
    let mut first = [0u32; 256];
    let mut sum = 0u32;
    for c in 0..256 {
        first[c] = sum;
        sum += counts[c];
    }
    let mut cur = first;
    let mut lf = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let j = cur[b as usize];
        cur[b as usize] += 1;
        lf[i] = j;
    }

    // Find a set of non-overlapping Baier tunnels in LF-direction:
    // A tunnel is a prefix-interval I_0 -> I_1 -> ... -> I_{h-1} of width w >= 2, height h >= 2
    // where I_0..I_{h-2} each lie in a single L-run, and all I_0..I_{h-1} are pairwise disjoint
    // and none of I_1..I_{h-2} contain p (or we exclude p from tunnels cleanly).
    // Greedily pick non-overlapping maximal-height tunnels for testing Baier's din/dout + stack inversion!
    let mut used = vec![false; n];
    // For Baier TBWT, we keep a mask `keep_row[x]` (false = removed interior/collapsed row),
    // `dout[x]` (true = start of a node's outgoing edges; false = extra collapsed outgoing edge),
    // `din[x]` (true = start of a node's incoming edges; false = extra collapsed incoming edge).
    // Specifically, in Baier (2018) Section 3:
    // For a disjoint tunnel I_0, I_1, ..., I_{h-1} of width w >= 2, height h >= 2 (h-1 LF steps):
    //   - At I_0 (entry under LF): keep all w entries in L, mark dout[s_0 + 1 .. s_0 + w] = false (merged into 1 node with out-degree w!)
    //   - Wait: in Baier's convention (walking BACKWARDS via LF):
    //     I_0 is the tunnel ENTRANCE (w rows -> 1 collapsed row, pushing offset t in 0..w-1 onto stack!),
    //     I_1 .. I_{h-2} have their top (or bottom) w-1 rows REMOVED completely (saving (h-2)*(w-1) rows, or (h-1)*(w-1) if I_0 is also collapsed in F and I_{h-1} in L!),
    //     Wait: let's implement Baier 2018 Algorithm 1 EXACTLY!
    //     In Baier 2018:
    //     - Entrance I_0 (width w): in F, all w rows stay (dout = 1 0 ... 0 for the w rows of I_0 -> wait: in LF direction,
    //       we arrive at row s_0 + t in I_0; we push offset t, and leave via the SINGLE representative row s_0!).
    //       So rows s_0+1 .. s_0+w-1 have their L-entries REMOVED (only s_0's L-entry remains, with out-degree 1 in L -> LF(s_0) = s_1!).
    //     - Intermediate I_1 .. I_{h-2}: rows s_k+1 .. s_k+w-1 are REMOVED in BOTH F and L! (1 row remains!)
    //     - Exit I_{h-1}: rows s_{h-1}+1 .. s_{h-1}+w-1 have their F-entries REMOVED (din = 1 0 ... 0), while all w L-entries remain!
    //       When LF arrives at the single F-node s_{h-1}, we pop offset t from stack and step to L-entry s_{h-1} + t!
    //     Let's check row counts saved per tunnel:
    //       L loses (h-1)*(w-1) entries! F loses (h-1)*(w-1) entries!
    //       Wait: notice I_0 has w entries in F (din=1..1, wait: dout=1,0..0) and 1 entry in L;
    //       I_{h-1} has 1 entry in F and w entries in L (din=1,0..0)!
    //       And EVERY remaining L-entry maps 1-to-1 via rank/C to a remaining F-entry!
    //     HOLD ON! Check if LF is STILL exact when we delete rows s_k+1..s_k+w-1 from L (for k=0..h-2) and from F (for k=1..h-1)!
    //       YES! Because for each k in 0..h-2, L[s_k..s_k+w] are ALL the same char c_k, AND F[s_{k+1}..s_{k+1}+w] are ALL the same char c_k!
    //       Deleting w-1 copies of c_k from L[I_k] and the EXACT SAME w-1 copies of c_k from F[I_{k+1}] preserves the
    //       order-preserving bijection between remaining L and remaining F!!
    let mut keep_l = vec![true; n];
    let mut keep_f = vec![true; n];
    let mut dout_f = vec![true; n]; // false for merged rows s_0+1..s_0+w-1 in F
    let mut din_l = vec![true; n];  // false for merged rows s_{h-1}+1..s_{h-1}+w-1 in L

    // Greedily find disjoint tunnels not containing p at any removed position:
    let mut i = 0;
    while i + 1 < n {
        if !used[i] && !used[i + 1] && l[i] == l[i + 1] {
            // Find maximal run width from i
            let c0 = l[i];
            let mut w = 1;
            while i + w < n && !used[i + w] && l[i + w] == c0 {
                w += 1;
            }
            // Trace LF while width stays >= 2 and all intervals disjoint and no removed row hits p
            let mut levels = vec![i];
            let mut cur_s = i;
            loop {
                let next_s = lf[cur_s] as usize;
                // Check disjointness from all previous levels and not already used
                let mut ok = true;
                for &prev_s in &levels {
                    if !(next_s + w <= prev_s || next_s >= prev_s + w) {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    for x in next_s..next_s + w {
                        if used[x] {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    break;
                }
                levels.push(next_s);
                // Can we continue another LF step? Need L[next_s..next_s+w] uniform!
                let cn = l[next_s];
                let mut uniform = true;
                for x in next_s + 1..next_s + w {
                    if l[x] != cn {
                        uniform = false;
                        break;
                    }
                }
                if !uniform {
                    break;
                }
                cur_s = next_s;
            }
            // Also check p is not in any level's removed rows (for simplicity exclude p from whole tunnel)
            let hits_p = levels.iter().any(|&s| p >= s && p < s + w);
            if levels.len() >= 2 && !hits_p {
                let h = levels.len();
                for &s in &levels {
                    for x in s..s + w {
                        used[x] = true;
                    }
                }
                // Apply Baier tunneling to levels[0..h]:
                // Entrance levels[0]: keep F[s0..s0+w] with dout_f[s0+1..s0+w] = false; remove L[s0+1..s0+w]
                let s0 = levels[0];
                for t in 1..w {
                    dout_f[s0 + t] = false;
                    keep_l[s0 + t] = false;
                }
                // Interior levels[1..h-1]: remove both F and L for t=1..w
                for &sk in &levels[1..h - 1] {
                    for t in 1..w {
                        keep_f[sk + t] = false;
                        keep_l[sk + t] = false;
                    }
                }
                // Exit levels[h-1]: remove F[sh+1..sh+w]; keep L[sh..sh+w] with din_l[sh+1..sh+w] = false
                let sh = levels[h - 1];
                for t in 1..w {
                    keep_f[sh + t] = false;
                    din_l[sh + t] = false;
                }
                i += w;
                continue;
            }
        }
        i += 1;
    }

    // Build compact TBWT arrays:
    let mut tl = Vec::new();
    let mut t_din = Vec::new();
    let mut t_dout = Vec::new();
    let mut f_pos_of_orig = vec![usize::MAX; n];
    let mut f_idx = 0usize;
    for x in 0..n {
        if keep_f[x] {
            f_pos_of_orig[x] = f_idx;
            t_dout.push(dout_f[x]);
            f_idx += 1;
        }
        if keep_l[x] {
            tl.push(l[x]);
            t_din.push(din_l[x]);
        }
    }
    let n_prime = tl.len();
    assert_eq!(n_prime, f_idx);
    let p_f = f_pos_of_orig[p];
    assert_ne!(p_f, usize::MAX);

    // Build rank/C on tl and node-index maps for t_dout / t_din:
    // Node index of F-pos x is rank_1(t_dout, x) - 1 (0-based node id); offset in node is x - select_1(node).
    // Node index of L-pos y is rank_1(t_din, y) - 1; starting L-pos of node v is select_1(t_din, v).
    let mut f_to_node = vec![0usize; n_prime];
    let mut f_to_off = vec![0usize; n_prime];
    let mut node_dout_deg = Vec::new();
    let mut cur_node = 0usize;
    let mut cur_start = 0usize;
    for x in 0..n_prime {
        if t_dout[x] {
            if x > 0 {
                node_dout_deg.push(x - cur_start);
                cur_node += 1;
            }
            cur_start = x;
        }
        f_to_node[x] = cur_node;
        f_to_off[x] = x - cur_start;
    }
    node_dout_deg.push(n_prime - cur_start);

    let mut node_to_lstart = Vec::new();
    let mut node_din_deg = Vec::new();
    let mut l_start = 0usize;
    for y in 0..n_prime {
        if t_din[y] {
            if y > 0 {
                node_din_deg.push(y - l_start);
            }
            l_start = y;
            node_to_lstart.push(y);
        }
    }
    node_din_deg.push(n_prime - l_start);
    assert_eq!(node_dout_deg.len(), node_to_lstart.len());

    // Build LF on tl (maps L-pos y -> F-pos lf_t[y])
    let mut t_counts = [0usize; 256];
    for &b in &tl { t_counts[b as usize] += 1; }
    let mut t_first = [0usize; 256];
    let mut s = 0;
    for c in 0..256 {
        t_first[c] = s;
        s += t_counts[c];
    }
    let mut t_cur = t_first;
    let mut lf_t = vec![0usize; n_prime];
    for (y, &b) in tl.iter().enumerate() {
        lf_t[y] = t_cur[b as usize];
        t_cur[b as usize] += 1;
    }

    // Invert backwards via LF using Baier's stack algorithm!
    // Start at F-pos = p_f (which corresponds to row p in original BWT).
    // Note: in backward LF walk starting from p, we recover T in REVERSE order:
    // L[p] = T[n-1], L[LF(p)] = T[n-2], ..., until returning to (p_f, empty stack) or n bytes!
    let mut rev_out = Vec::with_capacity(n);
    let mut stack: Vec<usize> = Vec::new();
    let mut fpos = p_f;
    let mut walk_steps = 0usize;
    for _ in 0..n {
        walk_steps += 1;
        let v = f_to_node[fpos];
        let off = f_to_off[fpos];
        if node_dout_deg[v] > 1 {
            stack.push(off);
        }
        let lpos = if node_din_deg[v] > 1 {
            let popped = stack.pop().expect("Baier stack underflow!");
            node_to_lstart[v] + popped
        } else {
            node_to_lstart[v]
        };
        let ch = tl[lpos];
        rev_out.push(ch);
        fpos = lf_t[lpos];
        if fpos == p_f && stack.is_empty() {
            break;
        }
    }
    rev_out.reverse();
    while rev_out.len() < n {
        let take = rev_out.len().min(n - rev_out.len());
        rev_out.extend_from_within(..take);
    }
    (rev_out, n_prime, walk_steps)
}

fn run_exhaustive_verifier() {
    println!("=== EXHAUSTIVE EXACTNESS VERIFIER ===");
    let mut bin_cases = 0usize;
    let mut bin_tbwt_collapsed = 0usize;
    let mut bin_lazy_hits = 0usize;
    for n in 1..=8usize {
        for mask in 0..(1usize << n) {
            let mut l = vec![0u8; n];
            for i in 0..n {
                l[i] = ((mask >> i) & 1) as u8;
            }
            for p in 0..n {
                let expected = plain_inverse(&l, p);
                // 1. Verify Baier TBWT exactness
                let (got_tbwt, n_prime, _wsteps) = baier_tbwt_invert_exact(&l, p);
                assert_eq!(got_tbwt, expected, "Baier TBWT mismatch on binary n={} mask={} p={}", n, mask, p);
                if n_prime < n {
                    bin_tbwt_collapsed += 1;
                }
                // 2. Verify LazyTunnel exactness across multiple limits (2, 4, 16) & min_steps (2, 3)
                for &lim in &[2usize, 4, 16] {
                    let om = build_open_map(&l, p, lim);
                    for &ms in &[2usize, 3] {
                        for &sub in &[false, true] {
                            let (got, st) = emit_lazy_tunnel(&om, n, ms, 16, sub);
                            assert_eq!(got, expected, "LazyTunnel mismatch on binary n={} mask={} p={} lim={} ms={} sub={}", n, mask, p, lim, ms, sub);
                            bin_lazy_hits += st.tunnel_hits;
                        }
                    }
                }
                bin_cases += 1;
            }
        }
    }
    println!("  [VERIFIED] Binary n<=8: {} (column, mark) cases PASSED (Baier TBWT collapsed in {} cases; LazyTunnel hit {} shortcuts)",
        bin_cases, bin_tbwt_collapsed, bin_lazy_hits);

    let mut ter_cases = 0usize;
    let mut ter_tbwt_collapsed = 0usize;
    let mut ter_lazy_hits = 0usize;
    for n in 1..=5usize {
        let total = 3usize.pow(n as u32);
        for mut code in 0..total {
            let mut l = vec![0u8; n];
            for i in 0..n {
                l[i] = (code % 3) as u8;
                code /= 3;
            }
            for p in 0..n {
                let expected = plain_inverse(&l, p);
                let (got_tbwt, n_prime, _) = baier_tbwt_invert_exact(&l, p);
                assert_eq!(got_tbwt, expected, "Baier TBWT mismatch on ternary n={} l={:?} p={}", n, l, p);
                if n_prime < n {
                    ter_tbwt_collapsed += 1;
                }
                for &lim in &[2usize, 4, 16] {
                    let om = build_open_map(&l, p, lim);
                    for &ms in &[2usize, 3] {
                        for &sub in &[false, true] {
                            let (got, st) = emit_lazy_tunnel(&om, n, ms, 16, sub);
                            assert_eq!(got, expected, "LazyTunnel mismatch on ternary n={} l={:?} p={} lim={} ms={} sub={}", n, l, p, lim, ms, sub);
                            ter_lazy_hits += st.tunnel_hits;
                        }
                    }
                }
                ter_cases += 1;
            }
        }
    }
    println!("  [VERIFIED] Ternary n<=5: {} (column, mark) cases PASSED (Baier TBWT collapsed in {} cases; LazyTunnel hit {} shortcuts)",
        ter_cases, ter_tbwt_collapsed, ter_lazy_hits);
}

fn main() {
    run_exhaustive_verifier();

    println!("\n=== 6-BLOCK EXACTNESS & LAZY-TUNNEL / BAIER-TBWT SIMULATION ===");
    let dir = Path::new("/tmp/mibz2-blocks");
    for b in BLOCKS {
        let l = fs::read(dir.join(b.file)).expect("read");
        let expected = plain_inverse(&l, b.p);
        let om = build_open_map(&l, b.p, 16);

        // Verify Classical Baier TBWT on full block!
        let (got_tbwt, n_prime, tbwt_steps) = baier_tbwt_invert_exact(&l, b.p);
        assert_eq!(got_tbwt, expected, "Baier TBWT mismatch on {}", b.file);

        println!("\n--- {} (n={}, m_16={}, words={}) ---", b.file, b.n, om.m, om.words.len());
        println!("  [Baier TBWT raw L] n'={} ({:.1}% of n, saved {:.1}% space) | walk_steps={} (= {:.1}% of n, {:.2}x m_16!)",
            n_prime, 100.0 * n_prime as f64 / b.n as f64,
            100.0 * (b.n - n_prime) as f64 / b.n as f64,
            tbwt_steps, 100.0 * tbwt_steps as f64 / b.n as f64,
            tbwt_steps as f64 / om.m as f64);

        for &(ms, sub) in &[(2, false), (2, true), (3, true), (4, true), (6, true), (8, true)] {
            let (got, st) = emit_lazy_tunnel(&om, b.n, ms, 16, sub);
            assert_eq!(got, expected, "LazyTunnel mismatch on {} ms={} sub={}", b.file, ms, sub);
            let saved = om.m - st.walk_steps;
            println!("  [LazyTunnel ms={} sub={:<5}] walk_Q={:<6} (saved {:<5} = {:4.1}% of m_16) | tunnels={:<5} writes={:<5} hits={:<5} bytes_by_tun={:<6} ({:4.1}% n) | chk/step={:.2}",
                ms, sub,
                st.walk_steps, saved, 100.0 * saved as f64 / om.m as f64,
                st.tunnels_recorded, st.shortcut_writes, st.tunnel_hits,
                st.bytes_by_tunnel, 100.0 * st.bytes_by_tunnel as f64 / b.n as f64,
                st.neighbor_checks as f64 / st.walk_steps as f64);
        }
    }
}
