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

/// Compute LF and Psi (inverse LF) for raw column L.
fn build_lf_psi(l: &[u8]) -> (Vec<u32>, Vec<u32>, [u32; 256]) {
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
    let mut lf = vec![0u32; n];
    let mut psi = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let j = cur[b as usize];
        cur[b as usize] += 1;
        lf[i] = j;
        psi[j as usize] = i as u32;
    }
    (lf, psi, first)
}

/// Maximal runs of L: (byte, start, width)
fn maximal_runs(l: &[u8]) -> Vec<(u8, usize, usize)> {
    let mut runs = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let c = l[i];
        let start = i;
        i += 1;
        while i < l.len() && l[i] == c {
            i += 1;
        }
        runs.push((c, start, i - start));
    }
    runs
}

/// Exact Open retained set S for a given limit (matches src/open.rs split logic!)
fn open_retained_mask(runs: &[(u8, usize, usize)], n: usize, p: usize, limit: usize) -> (Vec<bool>, Vec<u32>, Vec<u32>) {
    let mut is_ret = vec![false; n];
    for &(_c, pos, width) in runs {
        if width <= limit {
            for x in pos..pos + width {
                is_ret[x] = true;
            }
        } else if p >= pos && p < pos + width {
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
    (is_ret, ret_rows, row_to_rank)
}

/// Build Open's exact retained forward first-return map g: S -> S and word IDs.
/// Edge from u in S goes via Psi until next row in S.
/// Word spelled along u -> g(u) is F[u], F[Psi(u)], ..., F[Psi^{tau-1}(u)].
fn build_open_g(
    l: &[u8],
    psi: &[u32],
    first: &[u32; 256],
    is_ret: &[bool],
    ret_rows: &[u32],
    row_to_rank: &[u32],
) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let n = l.len();
    let m = ret_rows.len();
    // F[x] for each x
    let mut f_col = vec![0u8; n];
    for c in 0..256 {
        let start = first[c] as usize;
        let end = if c == 255 { n } else { first[c + 1] as usize };
        for x in start..end {
            f_col[x] = c as u8;
        }
    }

    // Assign exact word IDs (using FNV-1a / exact Vec<u8> map)
    use std::collections::HashMap;
    let mut word_map: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut g_next = vec![0u32; m];
    let mut g_code = vec![0u32; m];
    let mut g_wlen = vec![0u32; m];
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
        g_next[u] = row_to_rank[x];
        g_wlen[u] = buf.len() as u32;
        let code = if let Some(&id) = word_map.get(&buf) {
            id
        } else {
            let id = word_map.len() as u32;
            word_map.insert(buf.clone(), id);
            id
        };
        g_code[u] = code;
    }
    (g_next, g_code, g_wlen)
}

/// Analyze maximal 1-step strips and tunnels in g (width >= 2).
fn analyze_g_tunnels(
    g_next: &[u32],
    g_code: &[u32],
    g_wlen: &[u32],
    mark_rank: u32,
) {
    let m = g_next.len();
    // 1. Partition S into maximal 1-step strips of g:
    // adjacent u, u+1 are 1-step coupled iff g_code[u] == g_code[u+1] && g_next[u+1] == g_next[u] + 1
    let mut strip_id = vec![0u32; m];
    let mut strip_start = Vec::new();
    let mut strip_len = Vec::new();
    let mut u = 0;
    while u < m {
        let sid = strip_start.len() as u32;
        let start = u;
        strip_id[u] = sid;
        u += 1;
        while u < m && g_code[u] == g_code[u - 1] && g_next[u] == g_next[u - 1] + 1 {
            strip_id[u] = sid;
            u += 1;
        }
        strip_start.push(start as u32);
        strip_len.push((u - start) as u32);
    }

    let num_strips = strip_start.len();
    let mut strips_w1 = 0usize;
    let mut strips_wge2 = 0usize;
    let mut rows_in_wge2 = 0usize;
    for &w in &strip_len {
        if w == 1 {
            strips_w1 += 1;
        } else {
            strips_wge2 += 1;
            rows_in_wge2 += w as usize;
        }
    }

    // 2. Compute coupled-height array H[u] for u in 0..m-1:
    // H[u] = number of consecutive steps of g where (g^k(u), g^k(u+1)) are adjacent AND same strip!
    // Wait: we can compute H[u] in O(m) using the inverse of g (just like Kasai's LCP algorithm)
    // OR directly since coupled pairs satisfy: if u is coupled with u+1 (step 1), then H[u] = 1 + H[g[u]]!
    // Let's compute it cleanly using memoization / topological order (handling cycles safely).
    let mut h_arr = vec![-1i32; m.saturating_sub(1)];
    // coupled1[u] = true iff u and u+1 are in same 1-step strip
    let mut coupled1 = vec![false; m.saturating_sub(1)];
    for u in 0..m.saturating_sub(1) {
        if g_code[u] == g_code[u + 1] && g_next[u + 1] == g_next[u] + 1 {
            coupled1[u] = true;
        } else {
            h_arr[u] = 0;
        }
    }
    // For coupled1[u] == true, there is a functional edge u -> g_next[u] in [0..m-1).
    // Note: since g is a bijection, the functional graph on {u : coupled1[u]} has in-degree <= 1 and out-degree <= 1!
    // So it is a disjoint union of paths and cycles!
    let mut in_deg = vec![0u8; m.saturating_sub(1)];
    for u in 0..m.saturating_sub(1) {
        if coupled1[u] {
            let v = g_next[u] as usize;
            if v < m - 1 {
                in_deg[v] += 1;
            }
        }
    }
    let mut stack = Vec::new();
    // Process paths starting from nodes with coupled1[u] && in_deg[u] == 0
    for u0 in 0..m.saturating_sub(1) {
        if coupled1[u0] && in_deg[u0] == 0 {
            stack.clear();
            let mut cur = u0;
            while cur < m - 1 && coupled1[cur] && h_arr[cur] == -1 {
                stack.push(cur);
                cur = g_next[cur] as usize;
            }
            let mut base = if cur < m - 1 && h_arr[cur] >= 0 { h_arr[cur] } else { 0 };
            while let Some(x) = stack.pop() {
                base += 1;
                h_arr[x] = base;
            }
        }
    }
    // Any remaining h_arr[u] == -1 are on pure coupled cycles
    let mut cycle_pairs = 0usize;
    for u0 in 0..m.saturating_sub(1) {
        if h_arr[u0] == -1 {
            cycle_pairs += 1;
            h_arr[u0] = 1_000_000; // cap
        }
    }

    // Analyze H[u] distribution!
    let pairs_h_ge_1 = h_arr.iter().filter(|&&x| x >= 1).count();
    let pairs_h_ge_2 = h_arr.iter().filter(|&&x| x >= 2).count();
    let pairs_h_ge_5 = h_arr.iter().filter(|&&x| x >= 5).count();
    let pairs_h_ge_10 = h_arr.iter().filter(|&&x| x >= 10).count();
    let pairs_h_ge_20 = h_arr.iter().filter(|&&x| x >= 20).count();
    let max_h = h_arr.iter().copied().filter(|&x| x < 1_000_000).max().unwrap_or(0);

    // 3. EXACT COUNT OF ROWS IN NON-TRIVIAL TUNNELS OF g:
    // Notice: a row u in S belongs to SOME tunnel of width >= 2 and >= 2 steps of g (height >= 3)
    // iff either (u > 0 && H[u-1] >= 2) or (u < m-1 && H[u] >= 2), OR u is an intermediate/exit row of one!
    // Even better: in the functional graph on coupled pairs, EVERY coupled pair u -> g(u) with coupled1[u] == true
    // represents an edge of width >= 2.
    // Wait: what is the MAXIMUM POSSIBLE SAVINGS (ORACLE UPPER BOUND) of collapsing ALL width >= 2 strips in g?
    // Let's compute THREE exact quantities:
    // (Q1) "Maximal non-branching tunnel chains of 1-step strips of g" (exact Baier tunnels of g):
    //      For each row u in S, check if u is an INTERIOR row of a 2+ step tunnel of g:
    //      Specifically, u is an "internal tunnel row" if:
    //      - u is in a strip B_1 of width >= 2 (so u has a coupled neighbor in B_1), AND
    //      - g^{-1}(u) is ALSO in a strip B_0 of width >= 2 whose image g(B_0) shares that coupled neighbor with u!
    //      Wait: let's formalize this via the exact greedy / maximal tunnel partition AND exact strip-chain contraction!
    //
    // Let's trace ALL maximal strip-chains (exact maximal tunnels of g):
    // A strip B = [a, a+w) with w >= 2 maps to g(B) = [b, b+w).
    // If we contract ALL strips of width >= 2 in g (like Open(LIMIT=1) on g, or Rauzy-Veech on g's width>=2 strips):
    // Which rows of S are NOT in ANY strip of width >= 2?
    // That's `strips_w1`! Wait: EVERY row in `rows_in_wge2` is in a width >= 2 strip of g!
    // So if you eliminate ALL width >= 2 strips of g (leaving only width-1 rows of g + mark),
    // the remaining retained rows would be AT MOST `strips_w1 + 1`!
    //
    // NOW: what about STRICT BAIER TUNNELS (where the tunnel does NOT split — i.e., a block of width w >= 2
    // stays together for L >= 2 steps of g)?
    // Let's simulate:
    // (M1) **Greedy maximal-height non-overlapping tunnels in g** (exact oracle for disjoint shortcut tunnels):
    //      Start from entries where in_deg_coupled == 0 (or maximal blocks with H >= 2), trace how many rows are
    //      strictly interior (skipped by jumping from entry to exit)!
    // (M2) **Exact strip-to-strip chains (no splitting!)**:
    //      Strip A (width w >= 2) chains directly into Strip B iff g(A) is contained in B (even if B is wider!).
    //      Wait: if g(A) ⊆ B, then every row of A transitions to B with the SAME word W(B) and SAME shift!
    //      So A's rows can ALWAYS absorb B's step (contracting A -> B -> ... -> exit until g^k(A) splits across
    //      a strip boundary or hits a width-1 row or self-overlaps)!
    //      WAIT A SECOND! Let's measure BOTH:
    //      - Exact "non-splitting strip forward-contraction": from every strip A of width w >= 2 that is NOT
    //        fully inside g(A') for any strip A', trace A forward as long as g^k(A) stays inside a single strip!
    //      - Exact "lazy on-the-fly tunnel memoization" on the ACTUAL walk of `emit` from `mark_rank`!!
    //        THIS MEASURES THE EXACT NUMBER OF DEPENDENT LOADS SAVED ON THE REAL WALK!

    // Let's compute the exact lazy on-the-fly tunnel memoization upper bound AND exact strip-chain oracle!
    // First, let's check: how many steps in the ACTUAL cycle from mark_rank are in width>=2 strips,
    // and how many steps are saved by:
    //   (a) Oracle tunnel shortcutting (if every width>=2 strip B with g(B) ⊆ some strip is contracted to its exit)
    //   (b) Lazy on-the-fly tunnel shortcutting (first visit traces & caches, subsequent visits jump!)
    //   (c) Exact Baier prefix-interval count (sum over all maximal disjoint tunnels of (w-1)*(L-1) and w*(L-1))!

    // Let's simulate the EXACT forward-contraction of ALL width>=2 strips (non-splitting, no self-overlap):
    // A row u is "tunnel-internal" if there exists a strip B (width >= 2) such that g(B) contains u
    // AND g(B) is completely contained in strip_id[u] (i.e., the entire image of B stays coupled at u!).
    // Wait: what if two strips B1, B2 both have g(B1), g(B2) inside C? That's fine (they are disjoint in C since g is a bijection!).
    // Let's check: for each strip s with w = strip_len[s] >= 2, trace s forward while g^k(s) stays inside a single strip
    // (and does not contain mark_rank, and does not overlap earlier levels of the same trace).
    // Even simpler: define `can_skip[u]` = true iff:
    //   u != mark_rank AND
    //   u is in a strip of width >= 2 AND
    //   let p_u = g^{-1}(u); p_u is in a strip B_p of width >= 2 AND g(B_p) ⊆ strip_of(u)?
    // Wait: even sharper! For a specific row u, what if B_p has width 10, splits into 5+5 at g(B_p),
    // and the first 5 stay together for 30 more steps (a tunnel of width 5, height 30)?
    // That's a SUB-STRIP tunnel (Baier prefix-interval of width 5 inside a width-10 strip)!
    //
    // Let's measure BOTH:
    // 1. **Whole-strip tunnels** (no splitting of 1-step strips of g)
    // 2. **Arbitrary Baier sub-strip tunnels** (allowing splitting to any width w >= 2 with H >= 2!)

    // --- Measurement A: Arbitrary Baier sub-strip tunnels via H[u] ---
    // In the actual walk of g from mark_rank, every step u -> g(u) traverses an edge.
    // A step u -> g(u) is "interior to a Baier tunnel of height >= 3 (L >= 2 steps)" iff
    // the predecessor edge p_u -> u is coupled with at least one neighbor (so u has a neighbor v = u+1 or u-1
    // such that p_u, p_v are adjacent AND coupled, AND u, v are adjacent AND coupled!).
    // Wait: let's check that condition!
    // Pair (p_u, p_u + 1) has H[p_u] >= 2 iff:
    //   step 1: (p_u, p_u + 1) coupled -> maps to (u, u + 1)
    //   step 2: (u, u + 1) coupled -> maps to (g(u), g(u) + 1)
    // So u is the 2nd (or later) row of a width>=2, steps>=2 tunnel iff:
    //   (p_u < m-1 && H[p_u] >= 2) OR (p_u > 0 && H[p_u - 1] >= 2)  [wait: check exact index:
    //    if H[p_u] >= 2, then p_u and p_u+1 map to u and u+1, and BOTH u and u+1 are coupled!
    //    if H[p_u - 1] >= 2, then p_u-1 and p_u map to u-1 and u, and BOTH u-1 and u are coupled!]
    let mut g_inv = vec![0u32; m];
    for u in 0..m {
        g_inv[g_next[u] as usize] = u as u32;
    }

    let mut is_tunnel_interior_any_substrip = vec![false; m];
    let mut is_tunnel_entry_any_substrip = vec![false; m];
    for u in 0..m {
        let has_fwd2 = (u < m - 1 && h_arr[u] >= 2) || (u > 0 && h_arr[u - 1] >= 2);
        let pu = g_inv[u] as usize;
        let has_prev2 = (pu < m - 1 && h_arr[pu] >= 2) || (pu > 0 && h_arr[pu - 1] >= 2);
        if has_prev2 {
            // u is reached by a coupled step AND continues by a coupled step with the SAME neighbor!
            // Double check: if h_arr[pu] >= 2, then g(pu) = u and g(pu+1) = u+1, and h_arr[u] = h_arr[pu]-1 >= 1,
            // so u and u+1 ARE coupled!
            is_tunnel_interior_any_substrip[u] = true;
        }
        if has_fwd2 && !has_prev2 {
            is_tunnel_entry_any_substrip[u] = true;
        }
    }
    let interior_any = is_tunnel_interior_any_substrip.iter().filter(|&&b| b).count();
    let entry_any = is_tunnel_entry_any_substrip.iter().filter(|&&b| b).count();

    // --- Measurement B: Exact maximal disjoint tunnels (Baier Wheeler-graph edge savings) ---
    // In a coupled-pair chain of length L (edges e_0 -> e_1 -> ... -> e_{L-1} in the H-graph):
    // Wait: what is the EXACT number of rows saved by collapsing EVERY coupled adjacent pair (u, u+1)
    // that stays coupled for >= 2 steps?
    // Look at the H-graph: every node u in {0..m-2} with H[u] >= 2 has an outgoing coupled step to g(u)
    // which ALSO has H[g(u)] = H[u] - 1 >= 1!
    // Wait: if a tunnel has width w and L steps (so w-1 adjacent pairs, each of height L):
    //   - Number of pairs with H >= 2 inside this tunnel = (w - 1) * (L - 1)!!
    //   CHECK THAT FORMULA!!
    //   At level 0 (entry): w - 1 pairs have H = L >= 2
    //   At level 1:         w - 1 pairs have H = L - 1 >= 2 (if L >= 3)
    //   ...
    //   At level L - 2:     w - 1 pairs have H = 2
    //   At level L - 1:     w - 1 pairs have H = 1 (NOT >= 2!)
    //   TOTAL pairs with H >= 2 in this tunnel = (w - 1) * (L - 1)!!
    //   OMG!! WHAT AN IDENTITY!!
    //   Let's verify that!!
    //   Suppose a tunnel has width w and L steps of g (so it spans L+1 levels: entry J_0, interior J_1..J_{L-1}, exit J_L).
    //   - Oracle shortcutting (rewiring entry J_0 to jump straight to exit J_L):
    //     Saves w * (L - 1) steps (all rows in J_1..J_{L-1})!
    //   - Lazy 1st-visit memoization (1st occurrence walks L steps; other w-1 occurrences jump in 1 step):
    //     Saves (w - 1) * (L - 1) steps!!
    //   AND THAT EQUALS `pairs_h_ge_2` (the exact count of adjacent pairs u with H[u] >= 2)!!
    //   WAIT A SECOND!! IS `pairs_h_ge_2` REALLY THE EXACT SUM OF (w - 1) * (L - 1) OVER ALL NESTED TUNNELS?!
    //   LET'S CHECK WITH A NESTED TUNNEL!
    //   Suppose width 4 stays together for 1 step (L=1), then splits into 2+2, each staying together for 3 more steps:
    //   - Pair (0,1): H = 4 at level 0, 3 at level 1, 2 at level 2, 1 at level 3 -> 3 levels with H >= 2.
    //   - Pair (1,2): H = 1 at level 0 -> 0 levels with H >= 2.
    //   - Pair (2,3): H = 4 at level 0, 3 at level 1, 2 at level 2, 1 at level 3 -> 3 levels with H >= 2.
    //   Total pairs with H >= 2 = 3 + 0 + 3 = 6 = (2-1)*(4-1) + (2-1)*(4-1)!
    //   Wait: what if width 4 stays together for 2 steps, then splits into 2+2 for 2 more steps (total 4)?
    //   - Pair (0,1): H = 4, 3, 2, 1 -> 3 levels with H >= 2.
    //   - Pair (1,2): H = 2, 1 -> 1 level with H >= 2.
    //   - Pair (2,3): H = 4, 3, 2, 1 -> 3 levels with H >= 2.
    //   Total = 7!
    //   And how many steps does nested lazy memoization save?!
    //   - Occurrence 0 (left sub-tunnel): walks 4 steps (records width-4 tunnel for steps 0..2, and left width-2 for steps 2..4).
    //   - Occurrence 1 (left sub-tunnel): jumps 4 steps in 1 step -> saves 3 steps!
    //   - Occurrence 2 (right sub-tunnel): jumps steps 0..2 (saves 1 step!), then walks steps 2..4 (records right width-2).
    //   - Occurrence 3 (right sub-tunnel): jumps steps 0..2 (saves 1 step!), then jumps steps 2..4 (saves 1 step!) — or if merged, saves 3 steps!
    //     Wait: 3 + 1 + 2 = 6 if jumps don't merge, or 3 + 1 + 3 = 7 if they do!
    //   EXACTLY `pairs_h_ge_2` = 7!!
    //   WAIT A SECOND!! WHAT A THEOREM!!
    //   Let's check that algebraically:
    //   At every level, a maximal tunnel of width w > 1 has w-1 adjacent pairs.
    //   Every adjacent pair u with H[u] >= 2 means row u and row u+1 take the EXACT SAME step at this level AND the next level!
    //   So `pairs_h_ge_2` is an **EXACT UPPER BOUND** on lazy 1st-visit memoization savings (with ZERO pre-walk, even with arbitrary nesting!),
    //   AND `interior_any` is an **EXACT UPPER BOUND** on oracle shortcutting savings (with a free oracle that knows all nested tunnels)!

    // --- Measurement C: Whole-strip greedy chains (acquirable WITHOUT splitting strips!) ---
    // If we only chain whole 1-step strips of g: strip A chains to strip B iff g(A) ⊆ B.
    let mut strip_chains_next = vec![u32::MAX; num_strips];
    for s in 0..num_strips {
        let w = strip_len[s];
        if w < 2 { continue; }
        let a = strip_start[s] as usize;
        let b0 = g_next[a] as usize;
        let sb = strip_id[b0] as usize;
        let b_end = strip_start[sb] + strip_len[sb];
        if (b0 as u32) + w <= b_end && strip_len[sb] >= 2 {
            // whole strip s maps into strip sb!
            // check mark_rank not in g(s)
            if mark_rank < b0 as u32 || mark_rank >= b0 as u32 + w {
                strip_chains_next[s] = sb as u32;
            }
        }
    }
    let mut whole_strip_saved_oracle = 0usize;
    let mut whole_strip_saved_lazy = 0usize;
    let mut whole_strip_chains = 0usize;
    let mut whole_strip_trace_steps = 0usize;
    // Count how many rows are skipped if we contract all whole-strip chains s -> g(s) -> ...
    // Actually, for each strip s with w >= 2, if s has NO predecessor strip chaining onto its exact span,
    // Wait: even simpler! Count for each row u in a strip s (w>=2) whether s chains to another strip:
    // Actually, if strip A (width w) chains to strip B (width >= w), then every row of A skips 1 step in B!
    // Wait: if A -> B -> C (all width w), A skips B and C (2 steps per row of A)!
    // Wait: what if A (width 5) -> B (width 10) where B is NOT chained to C? Then A skips 1 step (B),
    // saving 5 steps (oracle) or 4 steps (lazy: trace 1 rep of A, skip 4)!
    // Wait: to avoid double counting when A -> B -> C (B's rows are ALSO visited by A, and maybe other rows of B not in g(A)):
    // Let's just simulate the ACTUAL walk on `mark_rank` with whole-strip shortcutting!
    let mut visited = vec![false; m];
    let mut cycle_len = 0usize;
    let mut cur = mark_rank as usize;
    loop {
        visited[cur] = true;
        cycle_len += 1;
        cur = g_next[cur] as usize;
        if cur == mark_rank as usize || visited[cur] {
            break;
        }
    }

    // Also count whole-strip chain edges: if strip s (width w >= 2) has strip_chains_next[s] != MAX,
    // then in the walk, whenever we are at row u in strip s, we can skip g(u) (moving directly to g(g(u))).
    // Let's mark every row v = g(u) that is skipped this way (careful not to skip a row that we start from,
    // i.e., compute exact shortcut jump table for whole-strip chains!).
    let mut strip_has_in = vec![false; num_strips];
    for s in 0..num_strips {
        if strip_chains_next[s] != u32::MAX {
            // Note: s might cover only a SUBSET of strip_chains_next[s]!
            // That's OK! If we shortcut strip s to follow chains until exit,
            // how many steps does the REAL walk from mark_rank take?
            let sb = strip_chains_next[s] as usize;
            if strip_len[s] == strip_len[sb] {
                strip_has_in[sb] = true;
            }
        }
    }
    // Build exact shortcut map for every row u in a strip with strip_chains_next[s] != MAX:
    // Follow chain from s (keeping width w = strip_len[s]) until it exits or hits mark or cycles.
    let mut sc_next = g_next.to_vec();
    let mut sc_steps = vec![1u32; m];
    let mut strip_is_head = vec![false; num_strips];
    for s in 0..num_strips {
        if strip_chains_next[s] != u32::MAX && !strip_has_in[s] {
            strip_is_head[s] = true;
            whole_strip_chains += 1;
            let w = strip_len[s];
            let a = strip_start[s];
            let mut x = a;
            let mut steps = 0u32;
            let mut seen_strips = std::collections::HashSet::new();
            loop {
                let cur_s = strip_id[x as usize];
                if !seen_strips.insert(cur_s) {
                    break;
                }
                let nx = g_next[x as usize];
                steps += 1;
                // Check if [nx, nx+w) is inside a single strip of width >= 2 and doesn't contain mark_rank
                let nsb = strip_id[nx as usize];
                let nsb_end = strip_start[nsb as usize] + strip_len[nsb as usize];
                let hits_mark = mark_rank >= nx && mark_rank < nx + w;
                if nx + w <= nsb_end && strip_len[nsb as usize] >= 2 && !hits_mark && steps < 10000 {
                    // Check we didn't overlap [a, a+w)
                    let overlaps_start = !(nx + w <= a || nx >= a + w);
                    if overlaps_start {
                        break;
                    }
                    x = nx;
                } else {
                    x = nx;
                    break;
                }
            }
            if steps >= 2 {
                whole_strip_trace_steps += steps as usize;
                for t in 0..w {
                    sc_next[(a + t) as usize] = x + t;
                    sc_steps[(a + t) as usize] = steps;
                }
            }
        }
    }
    // Walk using sc_next from mark_rank and count steps!
    let mut walk_steps_sc = 0usize;
    let mut walk_total_g_steps = 0usize;
    cur = mark_rank as usize;
    loop {
        walk_steps_sc += 1;
        walk_total_g_steps += sc_steps[cur] as usize;
        cur = sc_next[cur] as usize;
        if cur == mark_rank as usize || walk_total_g_steps >= cycle_len {
            break;
        }
    }
    whole_strip_saved_oracle = cycle_len.saturating_sub(walk_steps_sc);
    // Lazy simulation on the SAME whole-strip heads: first time a head strip is hit, pay full steps; next times pay 1!
    let mut head_seen = vec![false; num_strips];
    let mut lazy_steps = 0usize;
    let mut lazy_g_steps = 0usize;
    cur = mark_rank as usize;
    loop {
        let s = strip_id[cur] as usize;
        if sc_steps[cur] >= 2 && !head_seen[s] {
            head_seen[s] = true;
            lazy_steps += sc_steps[cur] as usize; // first visit traces all steps
        } else {
            lazy_steps += 1; // shortcut hit!
        }
        lazy_g_steps += sc_steps[cur] as usize;
        cur = sc_next[cur] as usize;
        if cur == mark_rank as usize || lazy_g_steps >= cycle_len {
            break;
        }
    }
    whole_strip_saved_lazy = cycle_len.saturating_sub(lazy_steps);

    println!("  [g-map] m_16={} (cycle_len={}) | 1-step strips: total={}, w=1: {} ({:.1}%), w>=2: {} ({} rows, {:.1}%)",
        m, cycle_len, num_strips, strips_w1, 100.0 * strips_w1 as f64 / m as f64,
        strips_wge2, rows_in_wge2, 100.0 * rows_in_wge2 as f64 / m as f64);
    println!("  [g-H-array] pairs H>=1: {} | H>=2 (EXACT nested lazy bound!): {} ({:.1}% of m) | H>=5: {} | H>=10: {} | H>=20: {} | maxH={} (cyc={})",
        pairs_h_ge_1, pairs_h_ge_2, 100.0 * pairs_h_ge_2 as f64 / m as f64,
        pairs_h_ge_5, pairs_h_ge_10, pairs_h_ge_20, max_h, cycle_pairs);
    println!("  [g-tunnels ANY sub-strip oracle] interior_rows (max oracle savings): {} ({:.1}% of m) | entry_rows: {}",
        interior_any, 100.0 * interior_any as f64 / m as f64, entry_any);
    println!("  [g-tunnels WHOLE-strip chains] heads={} | trace_steps={} | walk_steps_sc={} (saved oracle={} = {:.1}%, saved lazy={} = {:.1}%)",
        whole_strip_chains, whole_strip_trace_steps, walk_steps_sc,
        whole_strip_saved_oracle, 100.0 * whole_strip_saved_oracle as f64 / cycle_len as f64,
        whole_strip_saved_lazy, 100.0 * whole_strip_saved_lazy as f64 / cycle_len as f64);
}

/// Also measure raw Baier tunnels in L (for width >= 2 runs) and how many retained m_16 rows they touch!
fn analyze_raw_l_tunnels(
    l: &[u8],
    psi: &[u32],
    is_ret16: &[bool],
) {
    let n = l.len();
    // In Psi direction, adjacent rows (x, x+1) are 1-step coupled iff F[x]==F[x+1] && Psi[x+1] == Psi[x] + 1.
    // Wait: Psi[x+1] == Psi[x] + 1 happens iff L[Psi[x]] == L[Psi[x]+1] (same run in L!).
    // Equivalently in LF direction, (y, y+1) is 1-step coupled iff L[y] == L[y+1]!
    // Let's compute H_raw_psi[x] for x in 0..n-1: number of consecutive Psi steps where (x, x+1) stay adjacent in same F-bucket (i.e. same L-run of images)!
    let mut counts = [0u32; 256];
    for &b in l { counts[b as usize] += 1; }
    let mut f_col = vec![0u8; n];
    let mut pos = 0;
    for c in 0..256 {
        let cnt = counts[c] as usize;
        for x in pos..pos + cnt { f_col[x] = c as u8; }
        pos += cnt;
    }

    let mut coupled1 = vec![false; n - 1];
    let mut h_raw = vec![-1i32; n - 1];
    let mut in_deg = vec![0u8; n - 1];
    for x in 0..n - 1 {
        if f_col[x] == f_col[x + 1] && psi[x + 1] == psi[x] + 1 {
            coupled1[x] = true;
            let v = psi[x] as usize;
            if v < n - 1 {
                in_deg[v] += 1;
            }
        } else {
            h_raw[x] = 0;
        }
    }
    let mut stack = Vec::new();
    for x0 in 0..n - 1 {
        if coupled1[x0] && in_deg[x0] == 0 {
            stack.clear();
            let mut cur = x0;
            while cur < n - 1 && coupled1[cur] && h_raw[cur] == -1 {
                stack.push(cur);
                cur = psi[cur] as usize;
            }
            let mut base = if cur < n - 1 && h_raw[cur] >= 0 { h_raw[cur] } else { 0 };
            while let Some(z) = stack.pop() {
                base += 1;
                h_raw[z] = base;
            }
        }
    }
    for x0 in 0..n - 1 {
        if h_raw[x0] == -1 {
            h_raw[x0] = 1_000_000;
        }
    }

    // How many retained rows (is_ret16[x]) belong to a raw Baier tunnel (w>=2, h>=2, i.e., >=1 coupled Psi step)?
    // Specifically, a retained row x is inside a raw Baier tunnel of >= 2 steps (height >= 3) if
    // either x or x-1 has h_raw >= 2, OR LF(x) has h_raw >= 2!
    let mut ret_in_raw_h2 = 0usize;
    let mut ret_in_raw_h5 = 0usize;
    let mut ret_pairs_h2 = 0usize;
    let mut total_ret = 0usize;
    for x in 0..n {
        if is_ret16[x] {
            total_ret += 1;
            let h_here = (if x < n - 1 { h_raw[x] } else { 0 })
                .max(if x > 0 { h_raw[x - 1] } else { 0 });
            if h_here >= 2 { ret_in_raw_h2 += 1; }
            if h_here >= 5 { ret_in_raw_h5 += 1; }
            if x < n - 1 && is_ret16[x + 1] && h_raw[x] >= 2 {
                ret_pairs_h2 += 1;
            }
        }
    }
    let total_raw_pairs_h2 = h_raw.iter().filter(|&&h| h >= 2).count();
    println!("  [raw-L tunnels] total_pairs_H>=2: {} ({:.1}% of n) | ret16_rows in H_raw>=2: {} ({:.1}% of m_16), in H_raw>=5: {} ({:.1}% of m_16) | adj ret16 pairs H_raw>=2: {}",
        total_raw_pairs_h2, 100.0 * total_raw_pairs_h2 as f64 / n as f64,
        ret_in_raw_h2, 100.0 * ret_in_raw_h2 as f64 / total_ret as f64,
        ret_in_raw_h5, 100.0 * ret_in_raw_h5 as f64 / total_ret as f64,
        ret_pairs_h2);
}

fn main() {
    let dir = Path::new("/tmp/mibz2-blocks");
    println!("=== 6-BLOCK EXACT BWT TUNNELING & OPEN(g) CENSUS ===");
    for b in BLOCKS {
        let path = dir.join(b.file);
        let l = fs::read(&path).expect("read block");
        assert_eq!(l.len(), b.n);
        let runs = maximal_runs(&l);
        assert_eq!(runs.len(), b.r);

        println!("\n--- {} (n={}, p={}, r={}) ---", b.file, b.n, b.p, b.r);
        // Width sweep of Open retained rows m_L and thick strip count!
        let mut sweep = String::new();
        for &lim in &[1usize, 2, 4, 8, 16] {
            let mut m_lim = 0usize;
            let mut strips_lim = 0usize;
            for &(_, pos, w) in &runs {
                if w <= lim {
                    m_lim += w;
                } else if b.p >= pos && b.p < pos + w {
                    m_lim += 1;
                    if b.p > pos { strips_lim += 1; }
                    if b.p + 1 < pos + w { strips_lim += 1; }
                } else {
                    strips_lim += 1;
                }
            }
            sweep.push_str(&format!("L={}: m={} ({} strips) | ", lim, m_lim, strips_lim));
        }
        println!("  [Open limit sweep] {}", sweep);

        // Run width histogram for w = 1..=16
        let mut hist_runs = [0usize; 17];
        let mut hist_rows = [0usize; 17];
        for &(_, _, w) in &runs {
            if w <= 16 {
                hist_runs[w] += 1;
                hist_rows[w] += w;
            }
        }
        let w1_rows = hist_rows[1];
        let w2_16_rows: usize = hist_rows[2..=16].iter().sum();
        let w2_16_runs: usize = hist_runs[2..=16].iter().sum();
        println!("  [L thin runs <=16] w=1: {} rows | w=2..16: {} runs, {} rows ({:.1}% of thin)",
            w1_rows, w2_16_runs, w2_16_rows, 100.0 * w2_16_rows as f64 / (w1_rows + w2_16_rows) as f64);

        let (_lf, psi, first) = build_lf_psi(&l);
        let (is_ret16, ret_rows16, row_to_rank16) = open_retained_mask(&runs, b.n, b.p, 16);
        let mark_rank16 = row_to_rank16[b.p];
        let (g_next, g_code, g_wlen) = build_open_g(&l, &psi, &first, &is_ret16, &ret_rows16, &row_to_rank16);

        analyze_raw_l_tunnels(&l, &psi, &is_ret16);
        analyze_g_tunnels(&g_next, &g_code, &g_wlen, mark_rank16);
    }
}
