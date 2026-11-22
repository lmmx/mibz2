#![forbid(unsafe_code)]
//! Analogue B MSC2020 Catalogue Probe: MSC 55 (Algebraic Topology) & MSC 82 (Statistical Mechanics)
//! — Classical-First (1750s..1940s) + Modern (1950s..2000s) Tools (Cycle 46, P2).
//!
//! Computes and verifies on 100 small BWTs + Sturmian Fib(17,711) + 2 non-Wikidata synthetics + all 6 Wikidata blocks:
//!   MSC 55 (Algebraic Topology):
//!     - Classical Euler characteristic (1750s) & Poincare Betti curves beta_0(t), beta_1(t) (1890s)
//!     - Classical Whitehead elementary subdivision / width-1 deformation retract invariance (1930s)
//!     - Classical Mayer-Vietoris exact sequence on K_{>=1} = K_{>16} U K_{<=16} (1920s)
//!     - Modern Superlevel Persistent Homology Dgm_0(K_{>=t}) & exact F_2 Flag-Complex Dgm_1(Cl(K_{>=2})) (2000s)
//!   MSC 82 (Statistical Mechanics):
//!     - Classical Boltzmann-Gibbs canonical partition function Z(beta), C(beta) (1870s) & Pareto-Zipf alpha (1890s)
//!     - Classical Markov / Perron-Frobenius transfer operator T (1900s), exact stationarity pi_u = w_u/n,
//!       Kac return time (1940s), Kolmogorov-Sinai branching entropy h_KS(T) (1950s),
//!       Edwards-Anderson overlaps q_u^+, q_u^- (1970s), and Derrida REM Y_2(beta) (1980s)
//!     - Modern Broadbent-Hammersley Percolation P_inf(L), susceptibility chi_perc(L), L_c vs LIMIT=16 (1950s)
//!     - Modern 1D Migdal-Kadanoff Spin-Glass Valley unsupervised SHA1 classifier (precision/recall/F1) (1960s-1970s)

use std::collections::HashMap;
use std::fs;

const ADMITTED_DIR: &str = "/home/antigravity/admitted-bwt";

struct BlockSpec {
    name: &'static str,
    file: &'static str,
    expected_n: usize,
    primary_index: u32,
    expected_r: usize,
}

const WIKIDATA_BLOCKS: [BlockSpec; 6] = [
    BlockSpec { name: "block-0",   file: "block-0.L",   expected_n: 899_931, primary_index: 264_045, expected_r:  51_184 },
    BlockSpec { name: "block-2",   file: "block-2.L",   expected_n: 900_000, primary_index: 320_807, expected_r:  54_532 },
    BlockSpec { name: "block-85",  file: "block-85.L",  expected_n: 899_935, primary_index: 325_088, expected_r:  79_375 },
    BlockSpec { name: "block-143", file: "block-143.L", expected_n: 899_869, primary_index: 802_868, expected_r: 108_335 },
    BlockSpec { name: "block-207", file: "block-207.L", expected_n: 899_998, primary_index: 205_180, expected_r:  87_111 },
    BlockSpec { name: "block-210", file: "block-210.L", expected_n: 899_959, primary_index: 791_429, expected_r: 128_732 },
];

// ============================================================================
// 1. Core BWT, Run-Overlap Graph, & Diagnostic Text-Class Classifier (Audit Only)
// ============================================================================

#[derive(Clone, Copy, Debug)]
struct Run {
    start: u32,
    width: u32,
    img_start: u32,
}

fn counts_and_buckets(l: &[u8]) -> ([u32; 256], [u32; 256]) {
    let mut cnt = [0u32; 256];
    for &b in l {
        cnt[b as usize] += 1;
    }
    let mut c = [0u32; 256];
    let mut sum = 0u32;
    for i in 0..256 {
        c[i] = sum;
        sum += cnt[i];
    }
    (cnt, c)
}

fn plain_invert_with_pos(l: &[u8], orig_ptr: u32) -> (Vec<u8>, Vec<u32>) {
    let n = l.len();
    let (_cnt, mut cur) = counts_and_buckets(l);
    let mut psi = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let s = cur[b as usize] as usize;
        cur[b as usize] += 1;
        psi[s] = i as u32;
    }
    let mut out = vec![0u8; n];
    let mut pos_of_row = vec![u32::MAX; n];
    let mut row = orig_ptr;
    for t in 0..n {
        row = psi[row as usize];
        out[t] = l[row as usize];
        pos_of_row[row as usize] = t as u32;
    }
    (out, pos_of_row)
}

fn direct_bwt(t: &[u8]) -> (Vec<u8>, u32) {
    let n = t.len();
    let mut rot: Vec<usize> = (0..n).collect();
    rot.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = t[(a + k) % n];
            let cb = t[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut orig_ptr = 0u32;
    for (i, &start) in rot.iter().enumerate() {
        if start == 0 {
            orig_ptr = i as u32;
        }
        l.push(t[(start + n - 1) % n]);
    }
    (l, orig_ptr)
}

fn extract_runs(l: &[u8]) -> Vec<Run> {
    let (_cnt, mut cur) = counts_and_buckets(l);
    let mut runs = Vec::new();
    let n = l.len();
    let mut i = 0usize;
    while i < n {
        let b = l[i];
        let start = i as u32;
        while i < n && l[i] == b {
            i += 1;
        }
        let width = i as u32 - start;
        let img_start = cur[b as usize];
        cur[b as usize] += width;
        runs.push(Run { start, width, img_start });
    }
    runs
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum TextClass {
    Guid = 0,
    Sha1 = 1,
    JsonKey = 2,
    EntityId = 3,
    Number = 4,
    Text = 5,
    Structure = 6,
}

const CLASS_NAMES: [&str; 7] = [
    "GUID", "SHA1", "JSON-key", "entity-id", "number", "text", "structure",
];

fn is_hex(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

fn is_uuid36(s: &[u8]) -> bool {
    if s.len() < 36 {
        return false;
    }
    if s[8] != b'-' || s[13] != b'-' || s[18] != b'-' || s[23] != b'-' {
        return false;
    }
    for (i, &c) in s[..36].iter().enumerate() {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            continue;
        }
        if !is_hex(c) {
            return false;
        }
    }
    true
}

fn classify_text(text: &[u8]) -> Vec<TextClass> {
    let n = text.len();
    let mut cls = vec![TextClass::Structure; n];
    let mut assigned = vec![false; n];

    let mut i = 0usize;
    while i + 36 <= n {
        if is_uuid36(&text[i..i + 36]) {
            let mut start = i;
            if start >= 3 && text[start - 1] == b'$' {
                let mut j = start - 2;
                while j > 0 && text[j].is_ascii_digit() {
                    j -= 1;
                }
                if matches!(text[j], b'Q' | b'P' | b'L' | b'M') && j + 1 < start - 1 {
                    start = j;
                }
            }
            for k in start..(i + 36) {
                cls[k] = TextClass::Guid;
                assigned[k] = true;
            }
            i += 36;
        } else {
            i += 1;
        }
    }

    i = 0;
    while i + 40 <= n {
        if !assigned[i] && is_hex(text[i]) {
            let prev_ok = i == 0 || !is_hex(text[i - 1]);
            if prev_ok {
                let mut len = 0usize;
                while i + len < n && !assigned[i + len] && is_hex(text[i + len]) {
                    len += 1;
                }
                if len == 40 {
                    for k in i..(i + 40) {
                        cls[k] = TextClass::Sha1;
                        assigned[k] = true;
                    }
                    i += 40;
                    continue;
                } else {
                    i += len.max(1);
                    continue;
                }
            }
        }
        i += 1;
    }

    i = 0;
    while i < n {
        if text[i] == b'"' && !assigned[i] {
            let mut j = i + 1;
            while j < n && j - i <= 64 && text[j] != b'"' && text[j] != b'\n' && text[j] != b'\\' {
                j += 1;
            }
            if j < n && text[j] == b'"' && j + 1 < n && text[j + 1] == b':' {
                let mut any_assigned = false;
                for k in i..=(j + 1) {
                    if assigned[k] {
                        any_assigned = true;
                        break;
                    }
                }
                if !any_assigned && j > i + 1 {
                    for k in i..=(j + 1) {
                        cls[k] = TextClass::JsonKey;
                        assigned[k] = true;
                    }
                    i = j + 2;
                    continue;
                }
            }
        }
        i += 1;
    }

    i = 0;
    while i < n {
        if !assigned[i] && matches!(text[i], b'Q' | b'P' | b'L' | b'M') {
            let prev_ok = i == 0 || !text[i - 1].is_ascii_alphanumeric();
            if prev_ok && i + 1 < n && text[i + 1].is_ascii_digit() && !assigned[i + 1] {
                let mut j = i + 1;
                while j < n && text[j].is_ascii_digit() && !assigned[j] {
                    j += 1;
                }
                let next_ok = j == n || !text[j].is_ascii_alphanumeric();
                if next_ok {
                    for k in i..j {
                        cls[k] = TextClass::EntityId;
                        assigned[k] = true;
                    }
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }

    for k in 0..n {
        if !assigned[k] {
            if text[k].is_ascii_digit() {
                cls[k] = TextClass::Number;
            } else if text[k].is_ascii_alphabetic() || text[k] >= 0x80 {
                cls[k] = TextClass::Text;
            } else {
                cls[k] = TextClass::Structure;
            }
        }
    }
    cls
}

/// Exact directed and undirected run-overlap graph G_ov.
struct OverlapGraph {
    r: usize,
    widths: Vec<u32>,
    out_edges: Vec<Vec<(u32, u32)>>, // u -> (v, omega(u,v))
    in_edges: Vec<Vec<(u32, u32)>>,  // v -> (u, omega(u,v))
    dir_edge_count: usize,
    und_edges: Vec<(u32, u32, u32)>, // simple undirected {u < v, max_omega}
    self_loops: usize,
}

fn build_overlap_graph(runs: &[Run]) -> OverlapGraph {
    let r = runs.len();
    let widths: Vec<u32> = runs.iter().map(|x| x.width).collect();
    let mut out_edges = vec![Vec::new(); r];
    let mut in_edges = vec![Vec::new(); r];
    let mut dir_edge_count = 0usize;
    let mut und_map: HashMap<(u32, u32), u32> = HashMap::with_capacity(r * 2);
    let mut self_loops = 0usize;

    for (u, run) in runs.iter().enumerate() {
        let img_s = run.img_start;
        let img_e = img_s + run.width;
        let mut v = runs.partition_point(|x| x.start + x.width <= img_s);
        while v < r && runs[v].start < img_e {
            let ov_s = img_s.max(runs[v].start);
            let ov_e = img_e.min(runs[v].start + runs[v].width);
            let w = ov_e - ov_s;
            out_edges[u].push((v as u32, w));
            in_edges[v].push((u as u32, w));
            dir_edge_count += 1;
            if u == v {
                self_loops += 1;
            } else {
                let key = if u < v { (u as u32, v as u32) } else { (v as u32, u as u32) };
                let e = und_map.entry(key).or_insert(0);
                if w > *e {
                    *e = w;
                }
            }
            v += 1;
        }
    }

    let mut und_edges: Vec<(u32, u32, u32)> = und_map.into_iter().map(|((u, v), w)| (u, v, w)).collect();
    und_edges.sort_unstable_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)).then(a.1.cmp(&b.1)));

    OverlapGraph {
        r,
        widths,
        out_edges,
        in_edges,
        dir_edge_count,
        und_edges,
        self_loops,
    }
}

// ============================================================================
// 2. Disjoint Set Union (Union-Find) for Homology, Persistent H_0, & Percolation
// ============================================================================

struct Dsu {
    parent: Vec<u32>,
    birth: Vec<u32>,   // max run width in component (for Elder Rule)
    rep_run: Vec<u32>, // run id achieving birth
    row_mass: Vec<u64>,
    v_count: Vec<u32>,
}

impl Dsu {
    fn new(widths: &[u32]) -> Self {
        let n = widths.len();
        Self {
            parent: (0..n as u32).collect(),
            birth: widths.to_vec(),
            rep_run: (0..n as u32).collect(),
            row_mass: widths.iter().map(|&w| w as u64).collect(),
            v_count: vec![1; n],
        }
    }

    fn find(&mut self, mut x: u32) -> u32 {
        while self.parent[x as usize] != x {
            let p = self.parent[x as usize];
            self.parent[x as usize] = self.parent[p as usize];
            x = p;
        }
        x
    }

    /// Elder-rule union: younger (smaller birth width) merges into older (larger birth width).
    /// Returns Some((died_birth, died_rep_run, died_mass, new_mass)) if two distinct components merged.
    fn union_elder(&mut self, a: u32, b: u32) -> Option<(u32, u32, u64, u64)> {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return None;
        }
        let (older, younger) = if self.birth[ra as usize] > self.birth[rb as usize]
            || (self.birth[ra as usize] == self.birth[rb as usize] && ra < rb)
        {
            (ra, rb)
        } else {
            (rb, ra)
        };
        let died_birth = self.birth[younger as usize];
        let died_rep = self.rep_run[younger as usize];
        let died_mass = self.row_mass[younger as usize];
        self.parent[younger as usize] = older;
        self.row_mass[older as usize] += died_mass;
        self.v_count[older as usize] += self.v_count[younger as usize];
        Some((died_birth, died_rep, died_mass, self.row_mass[older as usize]))
    }
}

// ============================================================================
// 3. MSC 55: Algebraic Topology Diagnostics
//    (Euler 1750s, Poincare Betti 1890s, Mayer-Vietoris 1920s, Whitehead 1930s,
//     Persistent H_0 & Exact F_2 Flag-Complex Dgm_1 2000s)
// ============================================================================

struct TopoReport {
    // Classical Mayer-Vietoris on K_{>=1} = A (w > 16) U B (thin incident)
    beta0_all: usize,
    beta1_all: usize,
    beta0_a: usize,
    beta1_a: usize,
    beta0_b: usize,
    beta1_b: usize,
    beta0_inter: usize,
    rk_im_partial: usize,
    mv_exact: bool,
    // Classical Whitehead width-1 deformation retract
    w1_count: usize,
    w1_all_deg11: bool,
    w1_chains: usize,
    w1_chain_mean_len: f64,
    w1_guid_chains: usize,
    retract_exact: bool,
    // Betti curve at key scales t = 1, 2, 4, 8, 16, 17, 64, 256
    betti_samples: Vec<(u32, usize, usize, usize, usize)>, // (t, V, E, beta0, beta1)
    // Modern H_0 persistence (Elder rule)
    h0_finite_bars: usize,
    h0_pos_pers_bars: usize,      // birth > death
    h0_pos_pers_2_16: usize,      // born in 2..=16 with birth > death
    h0_pos_pers_17p: usize,       // born in >16 with birth > death
    h0_sha_pos_bars: usize,       // positive-persistence bars whose elder run is SHA1
    h0_max_ratio_2_16: f64,
    // Modern exact F_2 Flag-Complex Dgm_1(Cl(K_{>=2}))
    flag2_v: usize,
    flag2_e: usize,
    flag2_tri: usize,
    flag2_h1_born: usize,
    flag2_h1_killed: usize,
    flag2_h1_essential: usize,
    flag2_h1_pos_pers: usize,
}

fn graph_betti(v_count: usize, edges: &[(usize, usize)]) -> (usize, usize) {
    let mut parent: Vec<usize> = (0..v_count).collect();
    let find = |p: &mut [usize], mut x: usize| -> usize {
        while p[x] != x {
            let pp = p[x];
            p[x] = p[pp];
            x = pp;
        }
        x
    };
    let mut comps = v_count;
    for &(u, v) in edges {
        let ru = find(&mut parent, u);
        let rv = find(&mut parent, v);
        if ru != rv {
            parent[rv] = ru;
            comps -= 1;
        }
    }
    let beta1 = edges.len() + comps - v_count;
    (comps, beta1)
}

fn analyze_topology(g: &OverlapGraph, run_class: &[TextClass]) -> TopoReport {
    let r = g.r;
    // 1. Mayer-Vietoris on undirected simple graph K_{>=1} = A U B:
    //    A = induced subgraph on V_A = {u : w_u > 16} (thick crystal)
    //    B = all edges NOT inside A, together with all thin vertices (w <= 16) and their thick boundary neighbors!
    //    Then A ∩ B has ZERO edges and vertex set V(A) ∩ V(B)!
    let mut id_a = vec![usize::MAX; r];
    let mut va = 0usize;
    for u in 0..r {
        if g.widths[u] > 16 {
            id_a[u] = va;
            va += 1;
        }
    }
    let mut edges_a = Vec::new();
    let mut in_b = vec![false; r];
    for u in 0..r {
        if g.widths[u] <= 16 {
            in_b[u] = true;
        }
    }
    let mut raw_edges_b = Vec::new();
    let mut all_edges_usize = Vec::with_capacity(g.und_edges.len());
    for &(u, v, _) in &g.und_edges {
        let uu = u as usize;
        let vv = v as usize;
        all_edges_usize.push((uu, vv));
        if id_a[uu] != usize::MAX && id_a[vv] != usize::MAX {
            edges_a.push((id_a[uu], id_a[vv]));
        } else {
            in_b[uu] = true;
            in_b[vv] = true;
            raw_edges_b.push((uu, vv));
        }
    }
    let mut id_b = vec![usize::MAX; r];
    let mut vb = 0usize;
    let mut inter_v = 0usize;
    for u in 0..r {
        if in_b[u] {
            id_b[u] = vb;
            vb += 1;
            if id_a[u] != usize::MAX {
                inter_v += 1;
            }
        }
    }
    let edges_b: Vec<(usize, usize)> = raw_edges_b.iter().map(|&(u, v)| (id_b[u], id_b[v])).collect();

    let (beta0_all, beta1_all) = graph_betti(r, &all_edges_usize);
    let (beta0_a, beta1_a) = graph_betti(va, &edges_a);
    let (beta0_b, beta1_b) = graph_betti(vb, &edges_b);
    let beta0_inter = inter_v; // A ∩ B is 0-dimensional (no edges)
    let rk_im_partial = beta1_all - beta1_a - beta1_b;
    // Exact Mayer-Vietoris identity check:
    //   rk(im partial) == beta0(A∩B) - beta0(A) - beta0(B) + beta0(K)
    let mv_rhs = (beta0_inter as isize) - (beta0_a as isize) - (beta0_b as isize) + (beta0_all as isize);
    let mv_exact = (rk_im_partial as isize) == mv_rhs;

    // 2. Whitehead Width-1 Deformation Retract (on directed multigraph G_ov):
    //    Verify every w_u == 1 has deg^-(u) == deg^+(u) == 1!
    let mut w1_count = 0usize;
    let mut w1_all_deg11 = true;
    for u in 0..r {
        if g.widths[u] == 1 {
            w1_count += 1;
            if g.out_edges[u].len() != 1 || g.in_edges[u].len() != 1 {
                w1_all_deg11 = false;
            }
        }
    }
    // Count maximal directed width-1 chains a -> u_1 -> ... -> u_k -> b (with w_a >= 2, w_b >= 2)
    let mut w1_chains = 0usize;
    let mut w1_chain_nodes = 0usize;
    let mut w1_guid_chains = 0usize;
    let mut pure_w1_cycles = 0usize;
    let mut visited_w1 = vec![false; r];
    for u in 0..r {
        if g.widths[u] == 1 && !visited_w1[u] {
            let prev = g.in_edges[u][0].0 as usize;
            if g.widths[prev] >= 2 {
                // Start of maximal width-1 chain
                let mut cur = u;
                let mut len = 0usize;
                let mut guid_hits = 0usize;
                while g.widths[cur] == 1 && !visited_w1[cur] {
                    visited_w1[cur] = true;
                    len += 1;
                    if run_class[cur] == TextClass::Guid {
                        guid_hits += 1;
                    }
                    cur = g.out_edges[cur][0].0 as usize;
                }
                w1_chains += 1;
                w1_chain_nodes += len;
                if guid_hits * 2 >= len {
                    w1_guid_chains += 1;
                }
            }
        }
    }
    // Any remaining unvisited width-1 nodes must lie on pure width-1 cycles
    for u in 0..r {
        if g.widths[u] == 1 && !visited_w1[u] {
            pure_w1_cycles += 1;
            let mut cur = u;
            while !visited_w1[cur] {
                visited_w1[cur] = true;
                cur = g.out_edges[cur][0].0 as usize;
            }
        }
    }
    // Verify directed 1-complex Euler/Betti invariance under Whitehead contraction of all w=1 chains:
    // Original directed 1-complex: V = r, E = dir_edge_count, beta_0 = beta0_all (same underlying connectivity!)
    // Retracted directed 1-complex: removes `w1_chain_nodes` vertices and `w1_chain_nodes` edges (each chain of
    // length k has k vertices and k+1 edges, replaced by 1 direct edge -> delta V = -k, delta E = -k)!
    let v_ret = r - w1_chain_nodes;
    let e_ret = g.dir_edge_count - w1_chain_nodes;
    let beta1_dir_orig = g.dir_edge_count + beta0_all - r;
    let beta1_dir_ret = e_ret + beta0_all - v_ret;
    let retract_exact = w1_all_deg11 && (pure_w1_cycles == 0 || v_ret > 0) && (beta1_dir_orig == beta1_dir_ret);
    let w1_chain_mean_len = if w1_chains > 0 { w1_chain_nodes as f64 / w1_chains as f64 } else { 0.0 };

    // 3. Superlevel Filtration K_{>=t}: Betti curves + Persistent H_0 (Elder Rule)
    let mut verts_by_w: Vec<u32> = (0..r as u32).collect();
    verts_by_w.sort_unstable_by(|&a, &b| g.widths[b as usize].cmp(&g.widths[a as usize]).then(a.cmp(&b)));

    let mut dsu = Dsu::new(&g.widths);
    let mut v_idx = 0usize;
    let mut e_idx = 0usize;
    let mut active_v = 0usize;
    let mut active_e = 0usize;
    let mut comps = 0usize;

    let mut h0_finite_bars = 0usize;
    let mut h0_pos_pers_bars = 0usize;
    let mut h0_pos_pers_2_16 = 0usize;
    let mut h0_pos_pers_17p = 0usize;
    let mut h0_sha_pos_bars = 0usize;
    let mut h0_max_ratio_2_16 = 1.0f64;

    let sample_ts = [4096u32, 1024, 256, 64, 17, 16, 8, 4, 2, 1];
    let mut betti_samples = Vec::new();
    let mut s_ptr = 0usize;

    // Collect distinct thresholds in descending order
    let mut thresholds: Vec<u32> = g.widths.clone();
    thresholds.extend(g.und_edges.iter().map(|e| e.2));
    thresholds.extend(sample_ts.iter().copied());
    thresholds.sort_unstable_by(|a, b| b.cmp(a));
    thresholds.dedup();

    for &t in &thresholds {
        while v_idx < r && g.widths[verts_by_w[v_idx] as usize] >= t {
            active_v += 1;
            comps += 1;
            v_idx += 1;
        }
        while e_idx < g.und_edges.len() && g.und_edges[e_idx].2 >= t {
            let (u, v, w) = g.und_edges[e_idx];
            active_e += 1;
            if let Some((died_b, died_rep, _, _)) = dsu.union_elder(u, v) {
                comps -= 1;
                h0_finite_bars += 1;
                if died_b > w {
                    h0_pos_pers_bars += 1;
                    if died_b > 16 {
                        h0_pos_pers_17p += 1;
                    } else if died_b >= 2 {
                        h0_pos_pers_2_16 += 1;
                        let ratio = died_b as f64 / w as f64;
                        if ratio > h0_max_ratio_2_16 {
                            h0_max_ratio_2_16 = ratio;
                        }
                    }
                    if run_class[died_rep as usize] == TextClass::Sha1 {
                        h0_sha_pos_bars += 1;
                    }
                }
            }
            e_idx += 1;
        }
        while s_ptr < sample_ts.len() && sample_ts[s_ptr] >= t {
            if sample_ts[s_ptr] == t {
                let b1 = active_e + comps - active_v;
                betti_samples.push((t, active_v, active_e, comps, b1));
            }
            s_ptr += 1;
        }
    }
    betti_samples.reverse(); // ascending t = 1, 2, 4, ..., 4096

    // 4. Modern Exact F_2 Flag-Complex 1-Dimensional Persistent Homology Dgm_1(Cl(K_{>=2}))
    //    Vertices: w_u >= 2; Edges: W(u,v) >= 2; Triangles: {u,v,w} all 3 edges >= 2, birth = min(W_e).
    let mut id2 = vec![u32::MAX; r];
    let mut v2 = 0u32;
    for u in 0..r {
        if g.widths[u] >= 2 {
            id2[u] = v2;
            v2 += 1;
        }
    }
    let mut edges2: Vec<(u32, u32, u32)> = Vec::new();
    for &(u, v, w) in &g.und_edges {
        if w >= 2 {
            let a = id2[u as usize];
            let b = id2[v as usize];
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            edges2.push((lo, hi, w));
        }
    }
    // Sort edges in filtration order (descending weight W, deterministic tie-break)
    edges2.sort_unstable_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)).then(a.1.cmp(&b.1)));
    let e2_count = edges2.len();

    // Assign filtration index to each edge & build adjacency list oriented from lower to higher vertex id
    let mut adj_hi: Vec<Vec<(u32, u32, u32)>> = vec![Vec::new(); v2 as usize]; // u -> (v > u, edge_idx, weight)
    for (ei, &(u, v, w)) in edges2.iter().enumerate() {
        adj_hi[u as usize].push((v, ei as u32, w));
    }
    for lst in &mut adj_hi {
        lst.sort_unstable_by_key(|x| x.0);
    }

    // Enumerate all triangles {u < v < z} in K_{>=2} (fast sorted-intersection)
    // Each triangle stores (birth_weight, [e_uv, e_uz, e_vz]) with sorted edge indices!
    let mut triangles: Vec<(u32, [u32; 3])> = Vec::new();
    for u in 0..v2 as usize {
        let nu = &adj_hi[u];
        for i in 0..nu.len() {
            let (v, e_uv, w_uv) = nu[i];
            let nv = &adj_hi[v as usize];
            // Two-pointer intersection of nu[i+1..] and nv
            let mut p = i + 1;
            let mut q = 0usize;
            while p < nu.len() && q < nv.len() {
                let z1 = nu[p].0;
                let z2 = nv[q].0;
                if z1 == z2 {
                    let e_uz = nu[p].1;
                    let w_uz = nu[p].2;
                    let e_vz = nv[q].1;
                    let w_vz = nv[q].2;
                    let w_tri = w_uv.min(w_uz).min(w_vz);
                    let mut bdry = [e_uv, e_uz, e_vz];
                    bdry.sort_unstable();
                    triangles.push((w_tri, bdry));
                    p += 1;
                    q += 1;
                } else if z1 < z2 {
                    p += 1;
                } else {
                    q += 1;
                }
            }
        }
    }
    // Sort triangles in filtration order (descending weight w_tri, then lexicographic)
    triangles.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));

    // Compute exact H_1 births in K_{>=2} (edges that create cycles in Union-Find)
    let mut uf2: Vec<u32> = (0..v2).collect();
    let find2 = |p: &mut [u32], mut x: u32| -> u32 {
        while p[x as usize] != x {
            let pp = p[x as usize];
            p[x as usize] = p[pp as usize];
            x = pp;
        }
        x
    };
    let mut flag2_h1_born = 0usize;
    for &(u, v, _) in &edges2 {
        let ru = find2(&mut uf2, u);
        let rv = find2(&mut uf2, v);
        if ru != rv {
            uf2[rv as usize] = ru;
        } else {
            flag2_h1_born += 1;
        }
    }

    // Standard F_2 persistence boundary-matrix reduction on 2-simplices (columns = sorted Vec<u32> of edge indices)
    // Pivot = maximum edge index in column (latest-entering edge in filtration!)
    let mut low_to_col: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut flag2_h1_killed = 0usize;
    let mut flag2_h1_pos_pers = 0usize;

    let xor_sorted = |a: &[u32], b: &[u32]| -> Vec<u32> {
        let mut res = Vec::with_capacity(a.len() + b.len());
        let (mut i, mut j) = (0usize, 0usize);
        while i < a.len() && j < b.len() {
            if a[i] < b[j] {
                res.push(a[i]);
                i += 1;
            } else if b[j] < a[i] {
                res.push(b[j]);
                j += 1;
            } else {
                i += 1;
                j += 1;
            }
        }
        res.extend_from_slice(&a[i..]);
        res.extend_from_slice(&b[j..]);
        res
    };

    for &(w_tri, bdry) in &triangles {
        let mut col = bdry.to_vec();
        while let Some(&pivot) = col.last() {
            if let Some(other) = low_to_col.get(&pivot) {
                col = xor_sorted(&col, other);
            } else {
                let edge_birth_w = edges2[pivot as usize].2;
                flag2_h1_killed += 1;
                if edge_birth_w > w_tri {
                    flag2_h1_pos_pers += 1;
                }
                low_to_col.insert(pivot, col);
                break;
            }
        }
    }
    let flag2_h1_essential = flag2_h1_born - flag2_h1_killed;

    TopoReport {
        beta0_all,
        beta1_all,
        beta0_a,
        beta1_a,
        beta0_b,
        beta1_b,
        beta0_inter,
        rk_im_partial,
        mv_exact,
        w1_count,
        w1_all_deg11,
        w1_chains,
        w1_chain_mean_len,
        w1_guid_chains,
        retract_exact,
        betti_samples,
        h0_finite_bars,
        h0_pos_pers_bars,
        h0_pos_pers_2_16,
        h0_pos_pers_17p,
        h0_sha_pos_bars,
        h0_max_ratio_2_16,
        flag2_v: v2 as usize,
        flag2_e: e2_count,
        flag2_tri: triangles.len(),
        flag2_h1_born,
        flag2_h1_killed,
        flag2_h1_essential,
        flag2_h1_pos_pers,
    }
}

// ============================================================================
// 4. MSC 82: Statistical Mechanics Diagnostics
//    (Boltzmann-Gibbs 1870s, Pareto-Zipf 1890s, Markov/PF 1900s, Kac 1940s,
//     Percolation 1950s, KS Entropy 1950s, MK1D RG 1960s, EA/REM 1970s-80s)
// ============================================================================

struct StatMechReport {
    // Classical Markov stationarity & KS branching entropy
    stationarity_l1_err: f64,
    h_ks_full: f64,
    kac_mean_return_16: f64,
    // Edwards-Anderson overlaps per class (mean q_u^+ and q_u^-)
    ea_fwd_by_cls: [f64; 7],
    ea_bwd_by_cls: [f64; 7],
    // Classical Boltzmann-Gibbs / REM heat capacity peak & Zipf exponent
    zipf_alpha: f64,
    beta_c: f64,
    c_max: f64,
    y2_at_beta0: f64,
    y2_at_betac: f64,
    y2_at_neg1: f64,
    // Modern Broadbent-Hammersley Percolation on K_{>=L}
    lc_mass: u32,
    chi_max_mass: f64,
    p_inf_at_16: f64,
    p_inf_at_17: f64,
    p_inf_at_lc: f64,
    chi_at_16: f64,
    // 1D Migdal-Kadanoff Spin-Glass Valley Unsupervised SHA1 Classifier
    // (min_valley_len -> (precision%, recall%, f1%, valley_rows))
    mk1d_curve: Vec<(usize, f64, f64, f64, usize)>,
}

fn analyze_statmech(
    n: usize,
    g: &OverlapGraph,
    run_class: &[TextClass],
    pos_of_row: &[u32],
    row_cls: &[TextClass],
    runs: &[Run],
) -> StatMechReport {
    let r = g.r;
    let nf = n as f64;

    // 1. Classical Markov / Perron-Frobenius Transfer Operator T_{uv} = omega(u,v) / w_u
    //    Verify exact stationarity pi_u = w_u / n!
    let mut pi_t = vec![0.0f64; r];
    let mut h_ks_full = 0.0f64;
    let mut q_fwd = vec![0.0f64; r];
    let mut q_bwd = vec![0.0f64; r];
    let mut ea_fwd_sum = [0.0f64; 7];
    let mut ea_bwd_sum = [0.0f64; 7];
    let mut ea_wt_sum = [0.0f64; 7];

    for u in 0..r {
        let wu = g.widths[u] as f64;
        let pi_u = wu / nf;
        let mut s_u = 0.0f64;
        let mut qf = 0.0f64;
        for &(v, w) in &g.out_edges[u] {
            let p = w as f64 / wu;
            pi_t[v as usize] += pi_u * p;
            s_u -= p * p.log2();
            qf += p * p;
        }
        q_fwd[u] = qf;
        h_ks_full += pi_u * s_u;

        let mut qb = 0.0f64;
        for &(_src, w) in &g.in_edges[u] {
            let p = w as f64 / wu;
            qb += p * p;
        }
        q_bwd[u] = qb;

        let c = run_class[u] as usize;
        ea_fwd_sum[c] += wu * qf;
        ea_bwd_sum[c] += wu * qb;
        ea_wt_sum[c] += wu;
    }
    let mut stat_err = 0.0f64;
    for u in 0..r {
        stat_err += (pi_t[u] - g.widths[u] as f64 / nf).abs();
    }
    let mut ea_fwd_by_cls = [0.0f64; 7];
    let mut ea_bwd_by_cls = [0.0f64; 7];
    for c in 0..7 {
        if ea_wt_sum[c] > 0.0 {
            ea_fwd_by_cls[c] = ea_fwd_sum[c] / ea_wt_sum[c];
            ea_bwd_by_cls[c] = ea_bwd_sum[c] / ea_wt_sum[c];
        }
    }

    // Kac recurrence mean return time at LIMIT=16
    let thin_rows_16: usize = g.widths.iter().filter(|&&w| w <= 16).map(|&w| w as usize).sum();
    let kac_mean_return_16 = nf / thin_rows_16.max(1) as f64;

    // 2. Classical Boltzmann-Gibbs Z(beta) = sum w_u^{-beta} & REM Heat Capacity C(beta)
    //    + Pareto-Zipf MLE exponent alpha = 1 + N / sum ln(w / w_min) for w >= 2!
    let log_ws: Vec<f64> = g.widths.iter().map(|&w| (w as f64).ln()).collect();
    let mut zipf_n = 0usize;
    let mut zipf_sum_log = 0.0f64;
    for &w in &g.widths {
        if w >= 2 {
            zipf_n += 1;
            zipf_sum_log += (w as f64 / 1.5).ln();
        }
    }
    let zipf_alpha = if zipf_sum_log > 0.0 { 1.0 + zipf_n as f64 / zipf_sum_log } else { 0.0 };

    let eval_thermo = |beta: f64| -> (f64, f64) {
        // Numerically stable log-sum-exp for weights exp(-beta * ln w)
        let mut max_arg = f64::NEG_INFINITY;
        for &lw in &log_ws {
            let a = -beta * lw;
            if a > max_arg {
                max_arg = a;
            }
        }
        let mut z = 0.0f64;
        let mut z2 = 0.0f64;
        let mut mean_e = 0.0f64;
        let mut mean_e2 = 0.0f64;
        for &lw in &log_ws {
            let p = (-beta * lw - max_arg).exp();
            z += p;
            z2 += p * p;
            mean_e += p * lw;
            mean_e2 += p * lw * lw;
        }
        mean_e /= z;
        mean_e2 /= z;
        let var_e = (mean_e2 - mean_e * mean_e).max(0.0);
        let c_beta = beta * beta * var_e;
        let y2 = z2 / (z * z);
        (c_beta, y2)
    };

    let (_, y2_at_beta0) = eval_thermo(0.0);
    let (_, y2_at_neg1) = eval_thermo(-1.0);
    let mut beta_c = 0.0f64;
    let mut c_max = -1.0f64;
    let mut y2_at_betac = 0.0f64;
    for step in -40..=40 {
        let beta = step as f64 * 0.05;
        let (cb, y2) = eval_thermo(beta);
        if cb > c_max {
            c_max = cb;
            beta_c = beta;
            y2_at_betac = y2;
        }
    }

    // 3. Modern Broadbent-Hammersley Percolation on K_{>=L}
    //    Compute P_inf(L) and finite-cluster susceptibility chi_perc(L) = sum_{C != C_max} M(C)^2 / sum_{C != C_max} M(C)
    let mut verts_by_w: Vec<u32> = (0..r as u32).collect();
    verts_by_w.sort_unstable_by(|&a, &b| g.widths[b as usize].cmp(&g.widths[a as usize]).then(a.cmp(&b)));

    let mut dsu = Dsu::new(&g.widths);
    let mut v_idx = 0usize;
    let mut e_idx = 0usize;
    let mut total_active_mass = 0u64;
    let mut sum_m2: u128 = 0;
    let mut max_comp_mass = 0u64;

    let mut grid: Vec<u32> = (1..=64).collect();
    let mut t_val = 72u32;
    while t_val <= 65536 {
        grid.push(t_val);
        t_val = (t_val as f64 * 1.25) as u32;
    }
    grid.sort_unstable_by(|a, b| b.cmp(a));
    grid.dedup();

    let mut lc_mass = 1u32;
    let mut chi_max_mass = -1.0f64;
    let mut p_inf_at_16 = 0.0f64;
    let mut p_inf_at_17 = 0.0f64;
    let mut p_inf_at_lc = 0.0f64;
    let mut chi_at_16 = 0.0f64;

    for &t in &grid {
        while v_idx < r && g.widths[verts_by_w[v_idx] as usize] >= t {
            let u = verts_by_w[v_idx] as usize;
            let m = g.widths[u] as u64;
            total_active_mass += m;
            sum_m2 += (m as u128) * (m as u128);
            if m > max_comp_mass {
                max_comp_mass = m;
            }
            v_idx += 1;
        }
        while e_idx < g.und_edges.len() && g.und_edges[e_idx].2 >= t {
            let (u, v, _) = g.und_edges[e_idx];
            let ru = dsu.find(u);
            let rv = dsu.find(v);
            if ru != rv {
                let mu = dsu.row_mass[ru as usize] as u128;
                let mv = dsu.row_mass[rv as usize] as u128;
                sum_m2 -= mu * mu + mv * mv;
                let (_, _, _, new_m) = dsu.union_elder(u, v).unwrap();
                let nm = new_m as u128;
                sum_m2 += nm * nm;
                if new_m > max_comp_mass {
                    max_comp_mass = new_m;
                }
            }
            e_idx += 1;
        }
        let p_inf = max_comp_mass as f64 / nf;
        let fin_mass = total_active_mass.saturating_sub(max_comp_mass);
        let fin_m2 = sum_m2.saturating_sub((max_comp_mass as u128) * (max_comp_mass as u128));
        let chi = if fin_mass > 0 { fin_m2 as f64 / fin_mass as f64 } else { 0.0 };
        if t == 16 {
            p_inf_at_16 = p_inf;
            chi_at_16 = chi;
        }
        if t == 17 {
            p_inf_at_17 = p_inf;
        }
        if chi > chi_max_mass && t >= 2 {
            chi_max_mass = chi;
            lc_mass = t;
            p_inf_at_lc = p_inf;
        }
    }

    // 4. 1D Migdal-Kadanoff Spin-Glass Valley Classifier (2 <= w_u <= 16, q_u^+ == 1, q_u^- == 1)
    //    Exact-width chain length l_valley(u) purely from decoder-available G_ov!
    let mut is_iso_link = vec![false; r];
    let mut iso_next = vec![u32::MAX; r];
    let mut iso_prev = vec![u32::MAX; r];
    for u in 0..r {
        if g.widths[u] >= 2 && g.widths[u] <= 16 && g.out_edges[u].len() == 1 {
            let (v, w) = g.out_edges[u][0];
            if w == g.widths[u] && g.widths[v as usize] == g.widths[u] && g.in_edges[v as usize].len() == 1 {
                is_iso_link[u] = true;
                iso_next[u] = v;
                iso_prev[v as usize] = u as u32;
            }
        }
    }
    let mut valley_len = vec![0usize; r];
    let mut vis = vec![false; r];
    for u in 0..r {
        if is_iso_link[u] && !vis[u] && (iso_prev[u] == u32::MAX || !is_iso_link[iso_prev[u] as usize]) {
            let mut chain = Vec::new();
            let mut cur = u;
            while cur != u32::MAX as usize && is_iso_link[cur] && !vis[cur] {
                vis[cur] = true;
                chain.push(cur);
                let nx = iso_next[cur];
                cur = if nx == u32::MAX { usize::MAX } else { nx as usize };
            }
            let clen = chain.len();
            for &node in &chain {
                valley_len[node] = clen;
            }
        }
    }
    // Any remaining iso cycles
    for u in 0..r {
        if is_iso_link[u] && !vis[u] {
            let mut chain = Vec::new();
            let mut cur = u;
            while !vis[cur] {
                vis[cur] = true;
                chain.push(cur);
                cur = iso_next[cur] as usize;
            }
            let clen = chain.len();
            for &node in &chain {
                valley_len[node] = clen;
            }
        }
    }

    // Evaluate Precision / Recall / F1 against ground-truth SHA1 thin rows (w <= 16)
    let mut total_sha_thin = 0usize;
    for (u, run) in runs.iter().enumerate() {
        if g.widths[u] <= 16 {
            for row in run.start..(run.start + run.width) {
                if row_cls[pos_of_row[row as usize] as usize] == TextClass::Sha1 {
                    total_sha_thin += 1;
                }
            }
        }
    }
    let mut mk1d_curve = Vec::new();
    for &min_l in &[1usize, 4, 8, 16, 24, 32] {
        let mut pred_rows = 0usize;
        let mut tp_rows = 0usize;
        for (u, run) in runs.iter().enumerate() {
            if valley_len[u] >= min_l {
                for row in run.start..(run.start + run.width) {
                    pred_rows += 1;
                    if row_cls[pos_of_row[row as usize] as usize] == TextClass::Sha1 {
                        tp_rows += 1;
                    }
                }
            }
        }
        let prec = if pred_rows > 0 { 100.0 * tp_rows as f64 / pred_rows as f64 } else { 0.0 };
        let rec = if total_sha_thin > 0 { 100.0 * tp_rows as f64 / total_sha_thin as f64 } else { 0.0 };
        let f1 = if prec + rec > 0.0 { 2.0 * prec * rec / (prec + rec) } else { 0.0 };
        mk1d_curve.push((min_l, prec, rec, f1, pred_rows));
    }

    StatMechReport {
        stationarity_l1_err: stat_err,
        h_ks_full,
        kac_mean_return_16,
        ea_fwd_by_cls,
        ea_bwd_by_cls,
        zipf_alpha,
        beta_c,
        c_max,
        y2_at_beta0,
        y2_at_betac,
        y2_at_neg1,
        lc_mass,
        chi_max_mass,
        p_inf_at_16,
        p_inf_at_17,
        p_inf_at_lc,
        chi_at_16,
        mk1d_curve,
    }
}

// ============================================================================
// 5. Synthetic Generators (Matching Harness 1)
// ============================================================================

struct XorShift64(u64);
impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn gen_hex(rng: &mut XorShift64, len: usize) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    (0..len).map(|_| HEX[(rng.next() & 15) as usize]).collect()
}

fn gen_uuid(rng: &mut XorShift64) -> Vec<u8> {
    let h = gen_hex(rng, 32);
    let mut u = Vec::with_capacity(36);
    u.extend_from_slice(&h[0..8]);
    u.push(b'-');
    u.extend_from_slice(&h[8..12]);
    u.push(b'-');
    u.extend_from_slice(&h[12..16]);
    u.push(b'-');
    u.extend_from_slice(&h[16..20]);
    u.push(b'-');
    u.extend_from_slice(&h[20..32]);
    u
}

fn make_synthetic_json_log(num_records: usize) -> Vec<u8> {
    let mut rng = XorShift64(0x9E37_79B9_7F4A_7C15);
    let sha_pool: Vec<Vec<u8>> = (0..60).map(|_| gen_hex(&mut rng, 40)).collect();
    let endpoints = [b"/v2/items/sync".as_slice(), b"/v2/events/push", b"/v2/auth/verify", b"/v2/blobs/stat"];
    let levels = [b"INFO".as_slice(), b"WARN", b"DEBUG"];
    let mut out = Vec::with_capacity(num_records * 180);
    for i in 0..num_records {
        let uuid = gen_uuid(&mut rng);
        let sha = &sha_pool[(rng.next() as usize) % sha_pool.len()];
        let ep = endpoints[(rng.next() as usize) % endpoints.len()];
        let lv = levels[(rng.next() as usize) % levels.len()];
        out.extend_from_slice(b"{\"ts\":173225");
        out.extend_from_slice(format!("{:04}", i % 10_000).as_bytes());
        out.extend_from_slice(b",\"level\":\"");
        out.extend_from_slice(lv);
        out.extend_from_slice(b"\",\"req_id\":\"");
        out.extend_from_slice(&uuid);
        out.extend_from_slice(b"\",\"artifact_sha1\":\"");
        out.extend_from_slice(sha);
        out.extend_from_slice(b"\",\"endpoint\":\"");
        out.extend_from_slice(ep);
        out.extend_from_slice(b"\",\"status\":200}\n");
    }
    out
}

fn make_synthetic_syslog(num_lines: usize) -> Vec<u8> {
    let mut rng = XorShift64(0xD1B5_4A32_D192_ED03);
    let pkgs = [
        "libc6:amd64", "libssl3:amd64", "python3-minimal", "openssh-client",
        "systemd-sysv", "coreutils", "gcc-12-base", "zlib1g:amd64",
    ];
    let actions = ["status unpacked", "status half-configured", "status installed", "configure"];
    let mut out = Vec::with_capacity(num_lines * 90);
    for i in 0..num_lines {
        let p = pkgs[(rng.next() as usize) % pkgs.len()];
        let a = actions[i % actions.len()];
        let sec = i % 60;
        let min = (i / 60) % 60;
        out.extend_from_slice(
            format!("2026-11-22 04:{:02}:{:02} {} {} 1.{}.{}-ubuntu1\n", min, sec, a, p, i % 9, (rng.next() % 50)).as_bytes(),
        );
    }
    out
}

fn make_sturmian_column(p: usize, q: usize, a: u8, b: u8) -> Vec<u8> {
    let mut l = Vec::with_capacity(p + q);
    l.resize(q, b);
    l.resize(p + q, a);
    l
}

fn main() {
    println!("==========================================================================");
    println!("ANALOGUE B MSC2020 CATALOGUE: MSC 55 (TOPOLOGY) & MSC 82 (STAT MECHANICS)");
    println!("==========================================================================\n");

    // 1. Small-case theorem verification (100 random BWTs)
    let mut rng = XorShift64(0x1357_9BDF_2468_ACE0);
    let mut mv_ok = 0usize;
    let mut ret_ok = 0usize;
    let mut stat_ok = 0usize;
    for case in 0..100usize {
        let len = 40 + (rng.next() as usize % 80);
        let alpha = if case % 2 == 0 { 4u8 } else { 12u8 };
        let t: Vec<u8> = (0..len).map(|_| b'a' + (rng.next() as u8 % alpha)).collect();
        let (l, orig) = direct_bwt(&t);
        let (text, pos_of_row) = plain_invert_with_pos(&l, orig);
        let row_cls = classify_text(&text);
        let runs = extract_runs(&l);
        let g = build_overlap_graph(&runs);
        let run_cls: Vec<TextClass> = runs.iter().map(|r| row_cls[pos_of_row[r.start as usize] as usize]).collect();
        let tr = analyze_topology(&g, &run_cls);
        let sm = analyze_statmech(l.len(), &g, &run_cls, &pos_of_row, &row_cls, &runs);
        if tr.mv_exact {
            mv_ok += 1;
        }
        if tr.retract_exact {
            ret_ok += 1;
        }
        if sm.stationarity_l1_err < 1e-12 {
            stat_ok += 1;
        }
    }
    println!(
        "Small-Case Theorem Audit (100 random BWTs): Mayer-Vietoris exact={}/100, Whitehead w=1 retract exact={}/100, Markov stationarity ||pi*T-pi||_1<1e-12={}/100.\n",
        mv_ok, ret_ok, stat_ok
    );
    assert_eq!(mv_ok, 100);
    assert_eq!(ret_ok, 100);
    assert_eq!(stat_ok, 100);

    // 2. Load 6 Wikidata blocks + 2 Non-Wikidata Synthetics + Sturmian Fib(17,711)
    let mut blocks: Vec<(String, Vec<u8>, u32)> = Vec::new();
    for spec in &WIKIDATA_BLOCKS {
        let path = format!("{}/{}", ADMITTED_DIR, spec.file);
        let l = fs::read(&path).unwrap_or_else(|e| panic!("failed reading {path}: {e}"));
        assert_eq!(l.len(), spec.expected_n);
        assert_eq!(extract_runs(&l).len(), spec.expected_r);
        blocks.push((spec.name.to_string(), l, spec.primary_index));
    }
    let (syn_j_l, syn_j_p) = direct_bwt(&make_synthetic_json_log(480));
    blocks.push(("syn-json-log".to_string(), syn_j_l, syn_j_p));
    let (syn_s_l, syn_s_p) = direct_bwt(&make_synthetic_syslog(900));
    blocks.push(("syn-syslog".to_string(), syn_s_l, syn_s_p));
    blocks.push(("fib-sturm-17k".to_string(), make_sturmian_column(10_946, 6_765, b'a', b'b'), 4_000));

    // 3. MSC 55 Output
    println!("--- MSC 55 (ALGEBRAIC TOPOLOGY): Mayer-Vietoris, Whitehead Retract, Persistent H_0 & F_2 Flag Dgm_1 ---");
    for (name, l, orig) in &blocks {
        let (text, pos_of_row) = plain_invert_with_pos(l, *orig);
        let row_cls = classify_text(&text);
        let runs = extract_runs(l);
        let g = build_overlap_graph(&runs);
        let run_cls: Vec<TextClass> = runs.iter().map(|r| row_cls[pos_of_row[r.start as usize] as usize]).collect();
        let tr = analyze_topology(&g, &run_cls);
        assert!(tr.mv_exact, "Mayer-Vietoris failed on {name}");
        assert!(tr.retract_exact, "Whitehead retract failed on {name}");

        let pct_a = 100.0 * tr.beta1_a as f64 / tr.beta1_all.max(1) as f64;
        let pct_b = 100.0 * tr.beta1_b as f64 / tr.beta1_all.max(1) as f64;
        let pct_im = 100.0 * tr.rk_im_partial as f64 / tr.beta1_all.max(1) as f64;
        println!(
            "  {:<13} | r={:>6} E_dir={:>6} E_und={:>6} (self={}) | MV[EXACT]: b1(K)={:>5} = b1(A)={:>4} ({:>4.1}%) + b1(B)={:>5} ({:>4.1}%) + im(d)={:>5} ({:>4.1}%) [b0: K={},A={},B={},A∩B={}]",
            name, g.r, g.dir_edge_count, g.und_edges.len(), g.self_loops,
            tr.beta1_all, tr.beta1_a, pct_a, tr.beta1_b, pct_b, tr.rk_im_partial, pct_im,
            tr.beta0_all, tr.beta0_a, tr.beta0_b, tr.beta0_inter
        );
        println!(
            "     Whitehead w=1 [EXACT, deg11={}]: w1_runs={}, chains={} (mean_len={:.2}, guid_chains={} [{:.1}%])",
            tr.w1_all_deg11, tr.w1_count, tr.w1_chains, tr.w1_chain_mean_len, tr.w1_guid_chains,
            if tr.w1_chains > 0 { 100.0 * tr.w1_guid_chains as f64 / tr.w1_chains as f64 } else { 0.0 }
        );
        println!(
            "     Persist H_0: finite_bars={}, pos_pers(b>d)={} (2..16:{}, >16:{}, sha1_pos={}, max_b/d_2..16={:.1}x)",
            tr.h0_finite_bars, tr.h0_pos_pers_bars, tr.h0_pos_pers_2_16, tr.h0_pos_pers_17p, tr.h0_sha_pos_bars, tr.h0_max_ratio_2_16
        );
        println!(
            "     Flag Cl(K_>=2) F_2 Dgm_1: V2={}, E2={}, Tri2={}, H1_born={}, killed={} ({:.1}%, pos_pers={}), essential={}",
            tr.flag2_v, tr.flag2_e, tr.flag2_tri, tr.flag2_h1_born, tr.flag2_h1_killed,
            100.0 * tr.flag2_h1_killed as f64 / tr.flag2_h1_born.max(1) as f64,
            tr.flag2_h1_pos_pers, tr.flag2_h1_essential
        );
        print!("     Betti(t) [t:(b0,b1)]:");
        for &(t, _v, _e, b0, b1) in &tr.betti_samples {
            if matches!(t, 1 | 2 | 4 | 8 | 16 | 17 | 64 | 256 | 1024) {
                print!(" {}:({},{})", t, b0, b1);
            }
        }
        println!();
    }

    // 4. MSC 82 Output
    println!("\n--- MSC 82 (STATISTICAL MECHANICS): Percolation L_c, REM Z(beta)/C(beta), Transfer Operator T, MK1D SHA1 Classifier ---");
    for (name, l, orig) in &blocks {
        let (text, pos_of_row) = plain_invert_with_pos(l, *orig);
        let row_cls = classify_text(&text);
        let runs = extract_runs(l);
        let g = build_overlap_graph(&runs);
        let run_cls: Vec<TextClass> = runs.iter().map(|r| row_cls[pos_of_row[r.start as usize] as usize]).collect();
        let sm = analyze_statmech(l.len(), &g, &run_cls, &pos_of_row, &row_cls, &runs);
        assert!(sm.stationarity_l1_err < 1e-12, "Markov stationarity failed on {name}");

        println!(
            "  {:<13} | ||piT-pi||_1={:.1e} h_KS(T)={:.4} bits/step Kac<l16>={:.2} | Zipf_a={:.3} REM[b_c={:+.2}, C_max={:.2}, Y2(0)={:.2e}, Y2(bc)={:.4}, Y2(-1)={:.4}]",
            name, sm.stationarity_l1_err, sm.h_ks_full, sm.kac_mean_return_16,
            sm.zipf_alpha, sm.beta_c, sm.c_max, sm.y2_at_beta0, sm.y2_at_betac, sm.y2_at_neg1
        );
        println!(
            "     Percolation K_>=L: L_c={:>4} (chi_max={:>7.1}, P_inf(Lc)={:.3}) | at L=16: P_inf={:.3}, chi={:.1} | at L=17: P_inf={:.3}",
            sm.lc_mass, sm.chi_max_mass, sm.p_inf_at_lc, sm.p_inf_at_16, sm.chi_at_16, sm.p_inf_at_17
        );
        print!("     EA Overlap (q+/q-) by class:");
        for c in 0..7 {
            if sm.ea_fwd_by_cls[c] > 0.0 {
                print!(" {}={:.2}/{:.2}", CLASS_NAMES[c], sm.ea_fwd_by_cls[c], sm.ea_bwd_by_cls[c]);
            }
        }
        println!();
        print!("     MK1D Spin-Glass Valley -> SHA1 (Prec/Rec/F1 @min_len):");
        for &(ml, pr, rc, f1, rows) in &sm.mk1d_curve {
            print!(" [L>={}:P={:.1}%,R={:.1}%,F1={:.1}%,rows={}]", ml, pr, rc, f1, rows);
        }
        println!();
    }
}
