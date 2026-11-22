#![forbid(unsafe_code)]

use mibz2::bwt::InverseBwt;
use mibz2::runs::Runs;
use mibz2::Kernels;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    /// O(1)-depth threshold + 2-hash + parity with full bucket-split (Q_dep = 0 everywhere).
    HashThresh,
    /// Threshold + flat-array greedy Max-Cut with full bucket-split (25-26 rounds, ~4.2 r0 work).
    GreedyThresh,
    /// Pure threshold cut (zero bucket splits ever).
    ThreshOnly,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Run {
    pub sym: u32,
    pub len: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rule {
    /// Base byte: left == u32::MAX, right == byte
    /// Pair(a, b): left == a (bit 31 == 0), right == b
    /// Run(a, k): left == (0x8000_0000 | a), right == k
    pub left: u32,
    pub right: u32,
}

const RUN_TAG: u32 = 0x8000_0000;
const FINISH_TAG: u32 = 0x8000_0000;

#[derive(Clone, Copy, Debug, Default)]
pub struct BdmrStats {
    pub rounds: usize,
    pub self_rounds: usize,
    pub sum_r: u64,
    pub max_r: usize,
    pub max_sigma: usize,
    pub rules_count: usize,
    pub final_n: usize,
    /// Irregular scatter writes in counting-sort (Step 2.5) + greedy adj (if GreedyThresh)
    pub q_indep_scatter: u64,
    /// Irregular stream reads in Step 3 (c in Sigma_R + split c in Sigma_L)
    pub q_indep_stream: u64,
    /// Expansion tree visits (first-time rule expansions)
    pub q_expand_visits: u64,
    /// Expansion memoized extend_from_within copies
    pub q_expand_copies: u64,
    /// Peak scratch bytes across all vectors in BdmrKernel
    pub peak_scratch_bytes: usize,
    /// Stage timings (nanoseconds), populated only in instrumented runs
    pub acquire_ns: u64,
    pub contract_ns: u64,
    pub residual_ns: u64,
    pub expand_ns: u64,
    pub rotate_ns: u64,
}

pub struct BdmrKernel {
    pub mode: ColorMode,
    pub t_cut: u32,
    pub record_stats: bool,
    pub last_stats: BdmrStats,

    // Reusable flat scratch buffers (zero heap churn across blocks after warmup)
    runs: Vec<Run>,
    out_runs: Vec<Run>,
    counts: Vec<u32>,
    c_off: Vec<u32>,
    diff_asc: Vec<i64>,
    diff_desc: Vec<i64>,
    f_start: Vec<u32>,
    f_runs: Vec<Run>,    // (l_sym, len) sorted by f_sym
    f_new_id: Vec<u32>,  // pre-assigned new symbol ID per f_runs entry

    in_l: Vec<u8>,
    a_deg: Vec<u32>,
    a_off: Vec<u32>,
    by_a_eidx: Vec<u32>, // flat bucketed indices into f_runs
    by_a_fsym: Vec<u32>, // parallel f_sym for cache-friendly scan
    uncon_r_id: Vec<u32>,
    uncon_first_id: Vec<u32>,
    uncon_start: Vec<u32>,
    uncon_count: Vec<u32>,
    uncon_pieces: Vec<Run>, // (new_id, len)

    // Greedy Max-Cut flat adjacency (used only in GreedyThresh)
    g_deg: Vec<u32>,
    g_off: Vec<u32>,
    g_nb: Vec<u32>,
    g_wt: Vec<u32>,
    g_col: Vec<u8>,

    // Step 3 stream cursors
    b_idx: Vec<u32>,
    b_off: Vec<u32>,
    u_idx: Vec<u32>,
    u_off: Vec<u32>,

    // Grammar & symbol table
    rules: Vec<Rule>,
    sym_len: Vec<u32>,
    sym_to_rule: Vec<u32>,
    new_sym_to_rule: Vec<u32>,

    // Residual L1D walk scratch (<= t_cut)
    res_f: Vec<u32>,
    res_fl: Vec<u32>,
    res_pos: Vec<u32>,
    cycle_rules: Vec<u32>,

    // Memoized expansion scratch
    first_pos: Vec<u32>,
    exp_stack: Vec<(u32, u32)>,
}

impl Default for BdmrKernel {
    fn default() -> Self {
        Self::new(ColorMode::HashThresh, 4096)
    }
}

impl BdmrKernel {
    pub fn new(mode: ColorMode, t_cut: u32) -> Self {
        Self {
            mode,
            t_cut,
            record_stats: false,
            last_stats: BdmrStats::default(),
            runs: Vec::new(),
            out_runs: Vec::new(),
            counts: Vec::new(),
            c_off: Vec::new(),
            diff_asc: Vec::new(),
            diff_desc: Vec::new(),
            f_start: Vec::new(),
            f_runs: Vec::new(),
            f_new_id: Vec::new(),
            in_l: Vec::new(),
            a_deg: Vec::new(),
            a_off: Vec::new(),
            by_a_eidx: Vec::new(),
            by_a_fsym: Vec::new(),
            uncon_r_id: Vec::new(),
            uncon_first_id: Vec::new(),
            uncon_start: Vec::new(),
            uncon_count: Vec::new(),
            uncon_pieces: Vec::new(),
            g_deg: Vec::new(),
            g_off: Vec::new(),
            g_nb: Vec::new(),
            g_wt: Vec::new(),
            g_col: Vec::new(),
            b_idx: Vec::new(),
            b_off: Vec::new(),
            u_idx: Vec::new(),
            u_off: Vec::new(),
            rules: Vec::new(),
            sym_len: Vec::new(),
            sym_to_rule: Vec::new(),
            new_sym_to_rule: Vec::new(),
            res_f: Vec::new(),
            res_fl: Vec::new(),
            res_pos: Vec::new(),
            cycle_rules: Vec::new(),
            first_pos: Vec::new(),
            exp_stack: Vec::new(),
        }
    }

    pub fn scratch_bytes_allocated(&self) -> usize {
        self.runs.capacity() * 8
            + self.out_runs.capacity() * 8
            + self.counts.capacity() * 4
            + self.c_off.capacity() * 4
            + self.diff_asc.capacity() * 8
            + self.diff_desc.capacity() * 8
            + self.f_start.capacity() * 4
            + self.f_runs.capacity() * 8
            + self.f_new_id.capacity() * 4
            + self.in_l.capacity()
            + self.a_deg.capacity() * 4
            + self.a_off.capacity() * 4
            + self.by_a_eidx.capacity() * 4
            + self.by_a_fsym.capacity() * 4
            + self.uncon_r_id.capacity() * 4
            + self.uncon_first_id.capacity() * 4
            + self.uncon_start.capacity() * 4
            + self.uncon_count.capacity() * 4
            + self.uncon_pieces.capacity() * 8
            + self.g_deg.capacity() * 4
            + self.g_off.capacity() * 4
            + self.g_nb.capacity() * 4
            + self.g_wt.capacity() * 4
            + self.g_col.capacity()
            + self.b_idx.capacity() * 4
            + self.b_off.capacity() * 4
            + self.u_idx.capacity() * 4
            + self.u_off.capacity() * 4
            + self.rules.capacity() * 8
            + self.sym_len.capacity() * 4
            + self.sym_to_rule.capacity() * 4
            + self.new_sym_to_rule.capacity() * 4
            + self.res_f.capacity() * 4
            + self.res_fl.capacity() * 4
            + self.res_pos.capacity() * 4
            + self.cycle_rules.capacity() * 4
            + self.first_pos.capacity() * 4
            + self.exp_stack.capacity() * 8
    }

    fn current_live_bytes(&self) -> usize {
        self.runs.len() * 8
            + self.out_runs.len() * 8
            + self.counts.len() * 4
            + self.c_off.len() * 4
            + self.f_start.len() * 4
            + self.f_runs.len() * 8
            + self.f_new_id.len() * 4
            + self.by_a_eidx.len() * 4
            + self.by_a_fsym.len() * 4
            + self.uncon_pieces.len() * 8
            + self.rules.len() * 8
            + self.sym_len.len() * 4
            + self.sym_to_rule.len() * 4
            + self.first_pos.len() * 4
    }

    /// Core decode directly from capped bzip2 run records (I0).
    pub fn decode_records(&mut self, records: &[u16], n_orig: usize, p_init: u32, out: &mut Vec<u8>) {
        out.clear();
        if n_orig == 0 {
            return;
        }
        let rec_stats = self.record_stats;
        let mut st = BdmrStats::default();
        let t_acq = if rec_stats { Some(Instant::now()) } else { None };

        // 1. Acquire maximal runs & compact initial alphabet directly from capped records R
        let mut present = [0u32; 256];
        self.runs.clear();
        let mut i = 0;
        let rlen = records.len();
        while i < rlen {
            let rec = records[i];
            let c = (rec >> 8) as u8;
            let mut w = (rec & 255) as u32;
            i += 1;
            while i < rlen && (records[i] >> 8) as u8 == c {
                w += (records[i] & 255) as u32;
                i += 1;
            }
            present[c as usize] += w;
            self.runs.push(Run { sym: c as u32, len: w });
        }

        let mut remap = [0u32; 256];
        self.rules.clear();
        self.sym_len.clear();
        self.sym_to_rule.clear();
        let mut sigma: usize = 0;
        for byte in 0..256usize {
            if present[byte] > 0 {
                remap[byte] = sigma as u32;
                self.rules.push(Rule { left: u32::MAX, right: byte as u32 });
                self.sym_len.push(1);
                self.sym_to_rule.push(sigma as u32);
                sigma += 1;
            }
        }
        for run in &mut self.runs {
            run.sym = remap[run.sym as usize];
        }

        if let Some(t0) = t_acq {
            st.acquire_ns = t0.elapsed().as_nanos() as u64;
        }

        let t_con = if rec_stats { Some(Instant::now()) } else { None };

        let mut n = n_orig as u32;
        let mut p = p_init;
        let mut p_offset: u32 = 0;
        let mut single_root: Option<u32> = None;

        // 2. Contraction loop until fixed point or n <= t_cut
        loop {
            let r_cur = self.runs.len();
            if rec_stats {
                st.sum_r += r_cur as u64;
                if r_cur > st.max_r {
                    st.max_r = r_cur;
                }
                if sigma > st.max_sigma {
                    st.max_sigma = sigma;
                }
            }

            if n <= self.t_cut {
                if n == 1 {
                    single_root = Some(self.sym_to_rule[self.runs[0].sym as usize]);
                }
                break;
            }

            // 2a. Counts and prefix sums C[0..=sigma]
            self.counts.clear();
            self.counts.resize(sigma, 0);
            for run in &self.runs {
                self.counts[run.sym as usize] += run.len;
            }
            self.c_off.clear();
            self.c_off.resize(sigma + 1, 0);
            let mut tot = 0u32;
            for c in 0..sigma {
                self.c_off[c] = tot;
                tot += self.counts[c];
            }
            self.c_off[sigma] = tot;
            debug_assert_eq!(tot, n);

            // 2b. Single sequential pass: build f_runs + f_start + diff_asc/desc + locate p
            self.f_runs.clear();
            self.f_start.clear();
            self.f_start.resize(sigma + 1, 0);
            self.diff_asc.clear();
            self.diff_asc.resize(sigma + 1, 0);
            self.diff_desc.clear();
            self.diff_desc.resize(sigma + 1, 0);

            let mut pos = 0u32;
            let mut f_sym = 0usize;
            while f_sym < sigma && self.c_off[f_sym + 1] == 0 {
                f_sym += 1;
            }
            // Mark empty leading buckets
            for b in 0..=f_sym {
                self.f_start[b] = 0;
            }

            let mut self_total = 0u32;
            let mut p_l = 0u32;
            let mut p_f = 0u32;
            let mut p_rank = 0u32;
            // Reuse counts as seen_cnt (reset to 0)
            self.counts.fill(0);

            for idx in 0..r_cur {
                let run = self.runs[idx];
                let c = run.sym as usize;
                let w = run.len;
                if pos <= p && p < pos + w {
                    p_l = run.sym;
                    p_rank = self.counts[c] + (p - pos);
                }
                self.counts[c] += w;

                let mut rem = w;
                let mut cur_pos = pos;
                while rem > 0 {
                    while self.c_off[f_sym + 1] <= cur_pos {
                        f_sym += 1;
                        self.f_start[f_sym] = self.f_runs.len() as u32;
                    }
                    let avail = self.c_off[f_sym + 1] - cur_pos;
                    let take = rem.min(avail);
                    if cur_pos <= p && p < cur_pos + take {
                        p_f = f_sym as u32;
                    }
                    self.f_runs.push(Run { sym: run.sym, len: take });
                    let take_i = take as i64;
                    if c == f_sym {
                        self_total += take;
                    } else if c < f_sym {
                        self.diff_asc[c + 1] += take_i;
                        self.diff_asc[f_sym + 1] -= take_i;
                    } else {
                        self.diff_desc[f_sym + 1] += take_i;
                        self.diff_desc[c + 1] -= take_i;
                    }
                    cur_pos += take;
                    rem -= take;
                }
                pos += w;
            }
            let f_len = self.f_runs.len() as u32;
            for b in (f_sym + 1)..=sigma {
                self.f_start[b] = f_len;
            }

            if self_total == n {
                single_root = Some(self.sym_to_rule[p_f as usize]);
                break;
            }

            // 2c. Trigger exact 1-round self-run contraction if self-edges >= 5% of n
            if self_total > 0 && (self_total as u64) * 20 >= (n as u64) {
                if let Some(fp_sym) = self.do_self_run_contraction(sigma, n, &mut p, &mut p_offset) {
                    single_root = Some(self.sym_to_rule[fp_sym]);
                    break;
                }
                n -= self_total;
                sigma = self.sym_to_rule.len();
                st.rounds += 1;
                st.self_rounds += 1;
                continue;
            }

            // 2d. Choose partition Sigma_L U Sigma_R
            let (best_th_cnt, best_th_asc, best_th) = self.find_best_threshold(sigma);

            let (use_thresh, del_cnt) = match self.mode {
                ColorMode::ThreshOnly => {
                    if best_th_cnt == 0 {
                        if self_total > 0 {
                            if let Some(fp_sym) = self.do_self_run_contraction(sigma, n, &mut p, &mut p_offset) {
                                single_root = Some(self.sym_to_rule[fp_sym]);
                                break;
                            }
                            n -= self_total;
                            sigma = self.sym_to_rule.len();
                            st.rounds += 1;
                            st.self_rounds += 1;
                            continue;
                        } else {
                            single_root = Some(self.sym_to_rule[p_f as usize]);
                            break;
                        }
                    }
                    (true, best_th_cnt)
                }
                ColorMode::HashThresh => {
                    let (gen_cnt, use_gen) = self.choose_hash_coloring(sigma, st.rounds, best_th_cnt);
                    if use_gen {
                        (false, gen_cnt)
                    } else if best_th_cnt > 0 {
                        (true, best_th_cnt)
                    } else if self_total > 0 {
                        if let Some(fp_sym) = self.do_self_run_contraction(sigma, n, &mut p, &mut p_offset) {
                            single_root = Some(self.sym_to_rule[fp_sym]);
                            break;
                        }
                        n -= self_total;
                        sigma = self.sym_to_rule.len();
                        st.rounds += 1;
                        st.self_rounds += 1;
                        continue;
                    } else {
                        single_root = Some(self.sym_to_rule[p_f as usize]);
                        break;
                    }
                }
                ColorMode::GreedyThresh => {
                    let mc_cnt = self.choose_greedy_coloring(sigma, rec_stats, &mut st);
                    if best_th_cnt * 5 >= mc_cnt * 4 && best_th_cnt > 0 {
                        (true, best_th_cnt)
                    } else if mc_cnt > 0 {
                        (false, mc_cnt)
                    } else if self_total > 0 {
                        if let Some(fp_sym) = self.do_self_run_contraction(sigma, n, &mut p, &mut p_offset) {
                            single_root = Some(self.sym_to_rule[fp_sym]);
                            break;
                        }
                        n -= self_total;
                        sigma = self.sym_to_rule.len();
                        st.rounds += 1;
                        st.self_rounds += 1;
                        continue;
                    } else {
                        single_root = Some(self.sym_to_rule[p_f as usize]);
                        break;
                    }
                }
            };

            if use_thresh {
                self.in_l.clear();
                self.in_l.resize(sigma, 0);
                if best_th_asc {
                    for s in 0..best_th {
                        self.in_l[s] = 1;
                    }
                } else {
                    for s in best_th..sigma {
                        self.in_l[s] = 1;
                    }
                }
            }

            // 2e. Execute exact pair contraction (specialized fast path for zero-split threshold!)
            if use_thresh {
                self.execute_threshold_step(
                    sigma,
                    n,
                    best_th,
                    best_th_asc,
                    p_l,
                    p_f,
                    p_rank,
                    &mut p,
                    &mut p_offset,
                    rec_stats,
                    &mut st,
                );
            } else {
                self.execute_full_split_step(
                    sigma,
                    n,
                    p_l,
                    p_f,
                    p_rank,
                    &mut p,
                    &mut p_offset,
                    rec_stats,
                    &mut st,
                );
            }

            if rec_stats {
                let live = self.current_live_bytes();
                if live > st.peak_scratch_bytes {
                    st.peak_scratch_bytes = live;
                }
            }

            n -= del_cnt;
            sigma = self.sym_to_rule.len();
            st.rounds += 1;
        }

        if let Some(t0) = t_con {
            st.contract_ns = t0.elapsed().as_nanos() as u64;
        }
        st.final_n = n as usize;
        st.rules_count = self.rules.len();

        // 3. Residual cycle walk in L1D (if stopped early with n > 1)
        let t_res = if rec_stats { Some(Instant::now()) } else { None };
        self.cycle_rules.clear();
        if let Some(root) = single_root {
            self.cycle_rules.push(root);
        } else {
            let rn = n as usize;
            self.counts.clear();
            self.counts.resize(sigma, 0);
            for run in &self.runs {
                self.counts[run.sym as usize] += run.len;
            }
            self.res_pos.clear();
            self.res_pos.resize(sigma, 0);
            self.res_f.clear();
            self.res_f.resize(rn, 0);
            let mut tot = 0u32;
            for c in 0..sigma {
                self.res_pos[c] = tot;
                let cnt = self.counts[c] as usize;
                self.res_f[tot as usize..tot as usize + cnt].fill(c as u32);
                tot += self.counts[c];
            }
            self.res_fl.clear();
            self.res_fl.resize(rn, 0);
            let mut idx = 0u32;
            for run in &self.runs {
                let c = run.sym as usize;
                let base = self.res_pos[c] as usize;
                for off in 0..run.len {
                    self.res_fl[base + off as usize] = idx + off;
                }
                self.res_pos[c] += run.len;
                idx += run.len;
            }
            let mut cur = p;
            for _ in 0..rn {
                let fc = self.res_f[cur as usize] as usize;
                self.cycle_rules.push(self.sym_to_rule[fc]);
                cur = self.res_fl[cur as usize];
                if cur == p {
                    break;
                }
            }
        }
        if let Some(t0) = t_res {
            st.residual_ns = t0.elapsed().as_nanos() as u64;
        }

        // 4. Zero-buffer memoized SLP expansion directly into `out`!
        let t_exp = if rec_stats { Some(Instant::now()) } else { None };
        out.reserve(n_orig);
        self.first_pos.clear();
        self.first_pos.resize(self.rules.len(), u32::MAX);
        self.exp_stack.clear();

        let mut visits = 0u64;
        let mut copies = 0u64;

        for ci in 0..self.cycle_rules.len() {
            let top = self.cycle_rules[ci];
            self.exp_stack.push((top, 0));
            while let Some((tag_sym, start_pos)) = self.exp_stack.pop() {
                if (tag_sym & FINISH_TAG) != 0 {
                    let s = (tag_sym & !FINISH_TAG) as usize;
                    if self.first_pos[s] == u32::MAX {
                        self.first_pos[s] = start_pos;
                    }
                    continue;
                }
                let s = tag_sym as usize;
                let rl = self.rules[s];
                if rl.left == u32::MAX {
                    out.push(rl.right as u8);
                    continue;
                }
                let fp = self.first_pos[s];
                if fp != u32::MAX {
                    let fpu = fp as usize;
                    let len = self.sym_len[s] as usize;
                    out.extend_from_within(fpu..fpu + len);
                    if rec_stats {
                        copies += 1;
                    }
                    continue;
                }
                if rec_stats {
                    visits += 1;
                }
                let cur_out = out.len() as u32;
                if (rl.left & RUN_TAG) == 0 {
                    // Pair(left, right)
                    self.exp_stack.push((tag_sym | FINISH_TAG, cur_out));
                    self.exp_stack.push((rl.right, 0));
                    self.exp_stack.push((rl.left, 0));
                } else {
                    // Run(a, k): expand `a` once, then repeat via doubling extend_from_within!
                    let a = rl.left & !RUN_TAG;
                    let k = rl.right as usize;
                    // Expand `a` directly using a nested mini-loop if not memoized
                    self.expand_single_run(a, k, out, &mut visits, &mut copies, rec_stats);
                    self.first_pos[s] = cur_out;
                }
            }
        }
        if rec_stats {
            st.q_expand_visits = visits;
            st.q_expand_copies = copies;
            if let Some(t0) = t_exp {
                st.expand_ns = t0.elapsed().as_nanos() as u64;
            }
        }

        // 5. Rotate by p_offset and repeat if cycle length d < n_orig
        let t_rot = if rec_stats { Some(Instant::now()) } else { None };
        let d = out.len();
        debug_assert!(d > 0 && d <= n_orig);
        let rot = (p_offset as usize) % d;
        if rot != 0 {
            out.rotate_left(rot);
        }
        while out.len() < n_orig {
            let take = out.len().min(n_orig - out.len());
            out.extend_from_within(0..take);
        }
        if let Some(t0) = t_rot {
            st.rotate_ns = t0.elapsed().as_nanos() as u64;
        }
        if rec_stats {
            self.last_stats = st;
        }
    }

    fn expand_single_run(
        &mut self,
        a: u32,
        k: usize,
        out: &mut Vec<u8>,
        visits: &mut u64,
        copies: &mut u64,
        rec_stats: bool,
    ) {
        let start = out.len();
        let a_idx = a as usize;
        let rl = self.rules[a_idx];
        if rl.left == u32::MAX {
            let b = rl.right as u8;
            out.resize(start + k, b);
            return;
        }
        // Expand 1 copy of `a`
        let fp = self.first_pos[a_idx];
        let a_len = self.sym_len[a_idx] as usize;
        if fp != u32::MAX {
            let fpu = fp as usize;
            out.extend_from_within(fpu..fpu + a_len);
            if rec_stats {
                *copies += 1;
            }
        } else {
            let mut local_stack: Vec<(u32, u32)> = vec![(a, 0)];
            while let Some((tag_sym, sp)) = local_stack.pop() {
                if (tag_sym & FINISH_TAG) != 0 {
                    let s = (tag_sym & !FINISH_TAG) as usize;
                    if self.first_pos[s] == u32::MAX {
                        self.first_pos[s] = sp;
                    }
                    continue;
                }
                let s = tag_sym as usize;
                let r = self.rules[s];
                if r.left == u32::MAX {
                    out.push(r.right as u8);
                    continue;
                }
                let f = self.first_pos[s];
                if f != u32::MAX {
                    let fu = f as usize;
                    let ln = self.sym_len[s] as usize;
                    out.extend_from_within(fu..fu + ln);
                    if rec_stats {
                        *copies += 1;
                    }
                    continue;
                }
                if rec_stats {
                    *visits += 1;
                }
                let co = out.len() as u32;
                if (r.left & RUN_TAG) == 0 {
                    local_stack.push((tag_sym | FINISH_TAG, co));
                    local_stack.push((r.right, 0));
                    local_stack.push((r.left, 0));
                } else {
                    self.expand_single_run(r.left & !RUN_TAG, r.right as usize, out, visits, copies, rec_stats);
                    self.first_pos[s] = co;
                }
            }
        }
        // Doubling repeat to `k` copies of `a_len`
        let target_end = start + k * a_len;
        while out.len() < target_end {
            let cur_span = out.len() - start;
            let take = cur_span.min(target_end - out.len());
            out.extend_from_within(start..start + take);
        }
    }

    fn find_best_threshold(&self, sigma: usize) -> (u32, bool, usize) {
        let mut best_cnt = 0i64;
        let mut best_asc = true;
        let mut best_th = 0usize;
        let mut acc = 0i64;
        for th in 1..sigma {
            acc += self.diff_asc[th];
            if acc > best_cnt {
                best_cnt = acc;
                best_asc = true;
                best_th = th;
            }
        }
        acc = 0;
        for th in 1..sigma {
            acc += self.diff_desc[th];
            if acc > best_cnt {
                best_cnt = acc;
                best_asc = false;
                best_th = th;
            }
        }
        (best_cnt as u32, best_asc, best_th)
    }

    fn choose_hash_coloring(&mut self, sigma: usize, round: usize, best_th_cnt: u32) -> (u32, bool) {
        if sigma < 4 {
            return (0, false);
        }
        let m1: u32 = 2654435761;
        let m2: u32 = 2246822519;
        let s1 = ((round % 15) + 8) as u32;
        let s2 = (((round * 3 + 5) % 15) + 8) as u32;
        let r1 = (round as u32).wrapping_mul(17).wrapping_add(1);
        let r2 = (round as u32).wrapping_mul(31).wrapping_add(7);

        let mut c01 = [0u32; 3];
        let mut c10 = [0u32; 3];

        for b in 0..sigma {
            let bu = b as u32;
            let cb0 = (bu.wrapping_add(r1).wrapping_mul(m1) >> s1) & 1;
            let cb1 = (bu.wrapping_add(r2).wrapping_mul(m2) >> s2) & 1;
            let cb2 = bu & 1;
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let fr = self.f_runs[e];
                let a = fr.sym;
                if a == bu {
                    continue;
                }
                let w = fr.len;
                let ca0 = (a.wrapping_add(r1).wrapping_mul(m1) >> s1) & 1;
                if ca0 != cb0 {
                    if cb0 == 1 { c01[0] += w; } else { c10[0] += w; }
                }
                let ca1 = (a.wrapping_add(r2).wrapping_mul(m2) >> s2) & 1;
                if ca1 != cb1 {
                    if cb1 == 1 { c01[1] += w; } else { c10[1] += w; }
                }
                let ca2 = a & 1;
                if ca2 != cb2 {
                    if cb2 == 1 { c01[2] += w; } else { c10[2] += w; }
                }
            }
        }

        let mut best_cnt = 0u32;
        let mut best_kind = 0usize;
        let mut best_flip = 0u32;
        for k in 0..3 {
            if c01[k] > best_cnt {
                best_cnt = c01[k];
                best_kind = k;
                best_flip = 0;
            }
            if c10[k] > best_cnt {
                best_cnt = c10[k];
                best_kind = k;
                best_flip = 1;
            }
        }

        // Prefer zero-split threshold if >= 85% of best hash cut
        if (best_th_cnt as u64) * 20 >= (best_cnt as u64) * 17 && best_th_cnt > 0 {
            return (best_th_cnt, false);
        }
        if best_cnt == 0 {
            return (0, false);
        }

        self.in_l.clear();
        self.in_l.resize(sigma, 0);
        for s in 0..sigma {
            let su = s as u32;
            let col = match best_kind {
                0 => (su.wrapping_add(r1).wrapping_mul(m1) >> s1) & 1,
                1 => (su.wrapping_add(r2).wrapping_mul(m2) >> s2) & 1,
                _ => su & 1,
            };
            self.in_l[s] = u8::from((col ^ best_flip) == 0);
        }
        (best_cnt, true)
    }

    fn choose_greedy_coloring(&mut self, sigma: usize, rec_stats: bool, st: &mut BdmrStats) -> u32 {
        // Flat CSR adjacency to lower-indexed vertices (O(f_runs + sigma))
        self.g_deg.clear();
        self.g_deg.resize(sigma, 0);
        let mut total_edges = 0usize;
        for b in 0..sigma {
            let bu = b as u32;
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let a = self.f_runs[e].sym;
                if a == bu {
                    continue;
                }
                let hi_v = if a < bu { b } else { a as usize };
                self.g_deg[hi_v] += 1;
                total_edges += 1;
            }
        }
        self.g_off.clear();
        self.g_off.resize(sigma + 1, 0);
        let mut tot = 0u32;
        for s in 0..sigma {
            self.g_off[s] = tot;
            tot += self.g_deg[s];
        }
        self.g_off[sigma] = tot;
        self.g_nb.clear();
        self.g_nb.resize(total_edges, 0);
        self.g_wt.clear();
        self.g_wt.resize(total_edges, 0);
        // Use g_deg as write cursor
        self.g_deg.copy_from_slice(&self.g_off[..sigma]);
        for b in 0..sigma {
            let bu = b as u32;
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let fr = self.f_runs[e];
                let a = fr.sym;
                if a == bu {
                    continue;
                }
                let (hi_v, lo_v) = if a < bu { (b, a) } else { (a as usize, bu) };
                let dst = self.g_deg[hi_v] as usize;
                self.g_deg[hi_v] += 1;
                self.g_nb[dst] = lo_v;
                self.g_wt[dst] = fr.len;
            }
        }
        if rec_stats {
            st.q_indep_scatter += total_edges as u64;
        }

        self.g_col.clear();
        self.g_col.resize(sigma, 0);
        for s in 0..sigma {
            let lo = self.g_off[s] as usize;
            let hi = self.g_off[s + 1] as usize;
            let mut w0 = 0u64;
            let mut w1 = 0u64;
            for idx in lo..hi {
                let nb = self.g_nb[idx] as usize;
                let w = self.g_wt[idx] as u64;
                if self.g_col[nb] == 0 {
                    w0 += w;
                } else {
                    w1 += w;
                }
            }
            self.g_col[s] = u8::from(w0 >= w1);
        }

        let mut c01 = 0u32;
        let mut c10 = 0u32;
        for b in 0..sigma {
            let cb = self.g_col[b];
            let bu = b as u32;
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let fr = self.f_runs[e];
                if fr.sym != bu && self.g_col[fr.sym as usize] != cb {
                    if cb == 1 {
                        c01 += fr.len;
                    } else {
                        c10 += fr.len;
                    }
                }
            }
        }
        let flip = u8::from(c10 > c01);
        self.in_l.clear();
        self.in_l.resize(sigma, 0);
        for s in 0..sigma {
            self.in_l[s] = u8::from((self.g_col[s] ^ flip) == 0);
        }
        c01.max(c10)
    }

    /// Zero-split threshold pair contraction (O(r_t + sigma), zero uncon_pieces, L-runs 100% sequential!).
    #[allow(clippy::too_many_arguments)]
    fn execute_threshold_step(
        &mut self,
        sigma: usize,
        n: u32,
        th: usize,
        is_asc: bool,
        p_l: u32,
        p_f: u32,
        p_rank: u32,
        p: &mut u32,
        p_offset: &mut u32,
        rec_stats: bool,
        st: &mut BdmrStats,
    ) {
        let (l_lo, l_hi, r_lo, r_hi) = if is_asc {
            (0, th, th, sigma)
        } else {
            (th, sigma, 0, th)
        };

        // 1. Pre-assign new IDs via 1-pass counting sort of ONLY contracted f_runs (b in R, a in L)!
        self.a_deg.clear();
        self.a_deg.resize(sigma, 0);
        self.uncon_first_id.clear();
        self.uncon_first_id.resize(sigma, u32::MAX);
        // Track whether each symbol in L has any surviving uncontracted occurrence (in B_b for b in L)
        // and whether each symbol in R has any surviving uncontracted occurrence (in B_b for b in R with L in R)
        let mut l_has_uncon = vec![false; sigma];
        let mut r_has_uncon = vec![false; sigma];

        for b in l_lo..l_hi {
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let a = self.f_runs[e].sym as usize;
                if self.in_l[a] != 0 {
                    l_has_uncon[a] = true;
                }
            }
        }
        let mut contracted_e_cnt = 0usize;
        for b in r_lo..r_hi {
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let a = self.f_runs[e].sym as usize;
                if self.in_l[a] != 0 {
                    self.a_deg[a] += 1;
                    contracted_e_cnt += 1;
                } else {
                    r_has_uncon[b] = true;
                }
            }
        }

        self.a_off.clear();
        self.a_off.resize(sigma + 1, 0);
        let mut tot = 0u32;
        for s in 0..sigma {
            self.a_off[s] = tot;
            tot += self.a_deg[s];
        }
        self.a_off[sigma] = tot;
        self.by_a_eidx.clear();
        self.by_a_eidx.resize(contracted_e_cnt, 0);
        self.by_a_fsym.clear();
        self.by_a_fsym.resize(contracted_e_cnt, 0);
        self.a_deg.copy_from_slice(&self.a_off[..sigma]);

        for b in r_lo..r_hi {
            let bu = b as u32;
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let a = self.f_runs[e].sym as usize;
                if self.in_l[a] != 0 {
                    let dst = self.a_deg[a] as usize;
                    self.a_deg[a] += 1;
                    self.by_a_eidx[dst] = e as u32;
                    self.by_a_fsym[dst] = bu;
                }
            }
        }
        if rec_stats {
            st.q_indep_scatter += contracted_e_cnt as u64;
        }

        // Assign sorted new symbol IDs and fill f_new_id[e] directly!
        self.f_new_id.clear();
        self.f_new_id.resize(self.f_runs.len(), u32::MAX);
        self.new_sym_to_rule.clear();
        self.uncon_r_id.clear();
        self.uncon_r_id.resize(sigma, u32::MAX);

        for c in 0..sigma {
            if self.in_l[c] == 0 {
                if r_has_uncon[c] {
                    let nid = self.new_sym_to_rule.len() as u32;
                    self.uncon_r_id[c] = nid;
                    self.new_sym_to_rule.push(self.sym_to_rule[c]);
                }
            } else {
                let rc = self.sym_to_rule[c];
                let lo = self.a_off[c] as usize;
                let hi = self.a_off[c + 1] as usize;
                if is_asc {
                    // asc: uncontracted c comes BEFORE contracted (c, b)
                    if l_has_uncon[c] {
                        let nid = self.new_sym_to_rule.len() as u32;
                        self.uncon_first_id[c] = nid;
                        self.new_sym_to_rule.push(rc);
                    }
                    let mut idx = lo;
                    while idx < hi {
                        let b = self.by_a_fsym[idx];
                        let nid = self.new_sym_to_rule.len() as u32;
                        let rb = self.sym_to_rule[b as usize];
                        let rid = self.rules.len() as u32;
                        self.rules.push(Rule { left: rc, right: rb });
                        let len = self.sym_len[rc as usize] + self.sym_len[rb as usize];
                        self.sym_len.push(len);
                        self.new_sym_to_rule.push(rid);
                        while idx < hi && self.by_a_fsym[idx] == b {
                            self.f_new_id[self.by_a_eidx[idx] as usize] = nid;
                            idx += 1;
                        }
                    }
                } else {
                    // desc: contracted (c, b) comes BEFORE uncontracted c
                    let mut idx = lo;
                    while idx < hi {
                        let b = self.by_a_fsym[idx];
                        let nid = self.new_sym_to_rule.len() as u32;
                        let rb = self.sym_to_rule[b as usize];
                        let rid = self.rules.len() as u32;
                        self.rules.push(Rule { left: rc, right: rb });
                        let len = self.sym_len[rc as usize] + self.sym_len[rb as usize];
                        self.sym_len.push(len);
                        self.new_sym_to_rule.push(rid);
                        while idx < hi && self.by_a_fsym[idx] == b {
                            self.f_new_id[self.by_a_eidx[idx] as usize] = nid;
                            idx += 1;
                        }
                    }
                    if l_has_uncon[c] {
                        let nid = self.new_sym_to_rule.len() as u32;
                        self.uncon_first_id[c] = nid;
                        self.new_sym_to_rule.push(rc);
                    }
                }
            }
        }

        // Fill uncontracted entries in f_new_id for b in R
        for b in r_lo..r_hi {
            let ur = self.uncon_r_id[b];
            if ur == u32::MAX {
                continue;
            }
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                if self.f_new_id[e] == u32::MAX {
                    self.f_new_id[e] = ur;
                }
            }
        }

        // 2. Update p
        let new_p_old = if self.in_l[p_l as usize] != 0 && self.in_l[p_f as usize] == 0 {
            *p_offset += self.sym_len[self.sym_to_rule[p_l as usize] as usize];
            self.c_off[p_l as usize] + p_rank
        } else {
            *p
        };

        // 3. Emit out_runs in ONE pass!
        self.b_idx.clear();
        self.b_idx.extend_from_slice(&self.f_start[..sigma]);
        self.b_off.clear();
        self.b_off.resize(sigma, 0);
        self.out_runs.clear();

        let cut_pos = self.c_off[th];
        let mut pos = 0u32;
        let mut out_pos = 0u32;
        let mut new_p = u32::MAX;
        let mut stream_reads = 0u64;

        for r_idx in 0..self.runs.len() {
            let run = self.runs[r_idx];
            let c = run.sym as usize;
            let w = run.len;
            let end = pos + w;
            if self.in_l[c] != 0 {
                // PURE SEQUENTIAL: surviving slice is [pos, end) cap ([0, cut_pos) if asc else [cut_pos, n))
                let (s_lo, s_hi) = if is_asc {
                    (pos, end.min(cut_pos))
                } else {
                    (pos.max(cut_pos), end)
                };
                if s_lo < s_hi {
                    if s_lo <= new_p_old && new_p_old < s_hi {
                        new_p = out_pos + (new_p_old - s_lo);
                    }
                    let nid = self.uncon_first_id[c];
                    let take = s_hi - s_lo;
                    if let Some(last) = self.out_runs.last_mut() {
                        if last.sym == nid {
                            last.len += take;
                        } else {
                            self.out_runs.push(Run { sym: nid, len: take });
                        }
                    } else {
                        self.out_runs.push(Run { sym: nid, len: take });
                    }
                    out_pos += take;
                }
            } else {
                if pos <= new_p_old && new_p_old < end {
                    new_p = out_pos + (new_p_old - pos);
                }
                if rec_stats {
                    stream_reads += 1;
                }
                let mut bi = self.b_idx[c] as usize;
                let mut bo = self.b_off[c];
                let mut rem = w;
                while rem > 0 {
                    let blen = self.f_runs[bi].len;
                    let avail = blen - bo;
                    let take = rem.min(avail);
                    let nid = self.f_new_id[bi];
                    if let Some(last) = self.out_runs.last_mut() {
                        if last.sym == nid {
                            last.len += take;
                        } else {
                            self.out_runs.push(Run { sym: nid, len: take });
                        }
                    } else {
                        self.out_runs.push(Run { sym: nid, len: take });
                    }
                    out_pos += take;
                    rem -= take;
                    bo += take;
                    if bo == blen {
                        bi += 1;
                        bo = 0;
                    }
                }
                self.b_idx[c] = bi as u32;
                self.b_off[c] = bo;
            }
            pos = end;
        }
        debug_assert_ne!(new_p, u32::MAX);
        debug_assert!(out_pos < n);
        if rec_stats {
            st.q_indep_stream += stream_reads;
        }
        *p = new_p;
        std::mem::swap(&mut self.runs, &mut self.out_runs);
        std::mem::swap(&mut self.sym_to_rule, &mut self.new_sym_to_rule);
    }

    /// Full bucket-split pair contraction for arbitrary 2-coloring in_l[0..sigma).
    #[allow(clippy::too_many_arguments)]
    fn execute_full_split_step(
        &mut self,
        sigma: usize,
        n: u32,
        p_l: u32,
        p_f: u32,
        p_rank: u32,
        p: &mut u32,
        p_offset: &mut u32,
        rec_stats: bool,
        st: &mut BdmrStats,
    ) {
        // 1. 1-pass counting sort of ALL f_runs with l_sym in Sigma_L!
        self.a_deg.clear();
        self.a_deg.resize(sigma, 0);
        let mut r_has_uncon = vec![false; sigma];
        let mut l_e_cnt = 0usize;

        for b in 0..sigma {
            let b_in_l = self.in_l[b] != 0;
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let a = self.f_runs[e].sym as usize;
                if self.in_l[a] != 0 {
                    self.a_deg[a] += 1;
                    l_e_cnt += 1;
                } else if !b_in_l {
                    r_has_uncon[b] = true;
                }
            }
        }

        self.a_off.clear();
        self.a_off.resize(sigma + 1, 0);
        let mut tot = 0u32;
        for s in 0..sigma {
            self.a_off[s] = tot;
            tot += self.a_deg[s];
        }
        self.a_off[sigma] = tot;
        self.by_a_eidx.clear();
        self.by_a_eidx.resize(l_e_cnt, 0);
        self.by_a_fsym.clear();
        self.by_a_fsym.resize(l_e_cnt, 0);
        self.a_deg.copy_from_slice(&self.a_off[..sigma]);

        for b in 0..sigma {
            let bu = b as u32;
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let a = self.f_runs[e].sym as usize;
                if self.in_l[a] != 0 {
                    let dst = self.a_deg[a] as usize;
                    self.a_deg[a] += 1;
                    self.by_a_eidx[dst] = e as u32;
                    self.by_a_fsym[dst] = bu;
                }
            }
        }
        if rec_stats {
            st.q_indep_scatter += l_e_cnt as u64;
        }

        // 2. Sweep c = 0..sigma-1 to build sorted new IDs, f_new_id, and uncon_pieces!
        self.f_new_id.clear();
        self.f_new_id.resize(self.f_runs.len(), u32::MAX);
        self.new_sym_to_rule.clear();
        self.uncon_r_id.clear();
        self.uncon_r_id.resize(sigma, u32::MAX);
        self.uncon_first_id.clear();
        self.uncon_first_id.resize(sigma, u32::MAX);
        self.uncon_start.clear();
        self.uncon_start.resize(sigma, 0);
        self.uncon_count.clear();
        self.uncon_count.resize(sigma, 0);
        self.uncon_pieces.clear();

        for c in 0..sigma {
            if self.in_l[c] == 0 {
                if r_has_uncon[c] {
                    let nid = self.new_sym_to_rule.len() as u32;
                    self.uncon_r_id[c] = nid;
                    self.new_sym_to_rule.push(self.sym_to_rule[c]);
                }
            } else {
                let rc = self.sym_to_rule[c];
                let lo = self.a_off[c] as usize;
                let hi = self.a_off[c + 1] as usize;
                let u_st = self.uncon_pieces.len() as u32;
                self.uncon_start[c] = u_st;
                let mut idx = lo;
                while idx < hi {
                    let b = self.by_a_fsym[idx] as usize;
                    if self.in_l[b] != 0 {
                        // Contiguous uncontracted block of c!
                        let nid = self.new_sym_to_rule.len() as u32;
                        self.new_sym_to_rule.push(rc);
                        let mut ulen = 0u32;
                        while idx < hi && self.in_l[self.by_a_fsym[idx] as usize] != 0 {
                            ulen += self.f_runs[self.by_a_eidx[idx] as usize].len;
                            idx += 1;
                        }
                        if self.uncon_first_id[c] == u32::MAX {
                            self.uncon_first_id[c] = nid;
                        }
                        self.uncon_pieces.push(Run { sym: nid, len: ulen });
                    } else {
                        // Contracted pair (c, b)!
                        let bu = b as u32;
                        let nid = self.new_sym_to_rule.len() as u32;
                        let rb = self.sym_to_rule[b];
                        let rid = self.rules.len() as u32;
                        self.rules.push(Rule { left: rc, right: rb });
                        let len = self.sym_len[rc as usize] + self.sym_len[rb as usize];
                        self.sym_len.push(len);
                        self.new_sym_to_rule.push(rid);
                        while idx < hi && self.by_a_fsym[idx] == bu {
                            self.f_new_id[self.by_a_eidx[idx] as usize] = nid;
                            idx += 1;
                        }
                    }
                }
                self.uncon_count[c] = (self.uncon_pieces.len() as u32) - u_st;
            }
        }

        // Fill uncontracted entries in f_new_id for b in R
        for b in 0..sigma {
            if self.in_l[b] != 0 {
                continue;
            }
            let ur = self.uncon_r_id[b];
            if ur == u32::MAX {
                continue;
            }
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                if self.f_new_id[e] == u32::MAX {
                    self.f_new_id[e] = ur;
                }
            }
        }

        // 3. Update p
        let new_p_old = if self.in_l[p_l as usize] != 0 && self.in_l[p_f as usize] == 0 {
            *p_offset += self.sym_len[self.sym_to_rule[p_l as usize] as usize];
            self.c_off[p_l as usize] + p_rank
        } else {
            *p
        };

        // 4. Emit out_runs in ONE pass!
        self.b_idx.clear();
        self.b_idx.extend_from_slice(&self.f_start[..sigma]);
        self.b_off.clear();
        self.b_off.resize(sigma, 0);
        self.u_idx.clear();
        self.u_idx.extend_from_slice(&self.uncon_start[..sigma]);
        self.u_off.clear();
        self.u_off.resize(sigma, 0);
        self.out_runs.clear();

        let mut pos = 0u32;
        let mut out_pos = 0u32;
        let mut new_p = u32::MAX;
        let mut f_sym = 0usize;
        while f_sym < sigma && self.c_off[f_sym + 1] == 0 {
            f_sym += 1;
        }
        let mut stream_reads = 0u64;

        for r_idx in 0..self.runs.len() {
            let run = self.runs[r_idx];
            let c = run.sym as usize;
            let w = run.len;
            let end = pos + w;
            if self.in_l[c] != 0 {
                let split = self.uncon_count[c] > 1;
                let single_nid = self.uncon_first_id[c];
                let mut ui = self.u_idx[c] as usize;
                let mut uo = self.u_off[c];
                let mut rem = w;
                let mut cur_pos = pos;
                let mut touched_split = false;
                while rem > 0 {
                    while self.c_off[f_sym + 1] <= cur_pos {
                        f_sym += 1;
                    }
                    let avail = self.c_off[f_sym + 1] - cur_pos;
                    let take = rem.min(avail);
                    if self.in_l[f_sym] != 0 {
                        if cur_pos <= new_p_old && new_p_old < cur_pos + take {
                            new_p = out_pos + (new_p_old - cur_pos);
                        }
                        if !split {
                            if let Some(last) = self.out_runs.last_mut() {
                                if last.sym == single_nid {
                                    last.len += take;
                                } else {
                                    self.out_runs.push(Run { sym: single_nid, len: take });
                                }
                            } else {
                                self.out_runs.push(Run { sym: single_nid, len: take });
                            }
                            out_pos += take;
                        } else {
                            touched_split = true;
                            let mut t_rem = take;
                            while t_rem > 0 {
                                let piece = self.uncon_pieces[ui];
                                let p_avail = piece.len - uo;
                                let step = t_rem.min(p_avail);
                                let nid = piece.sym;
                                if let Some(last) = self.out_runs.last_mut() {
                                    if last.sym == nid {
                                        last.len += step;
                                    } else {
                                        self.out_runs.push(Run { sym: nid, len: step });
                                    }
                                } else {
                                    self.out_runs.push(Run { sym: nid, len: step });
                                }
                                out_pos += step;
                                t_rem -= step;
                                uo += step;
                                if uo == piece.len {
                                    ui += 1;
                                    uo = 0;
                                }
                            }
                        }
                    }
                    cur_pos += take;
                    rem -= take;
                }
                if split {
                    self.u_idx[c] = ui as u32;
                    self.u_off[c] = uo;
                    if rec_stats && touched_split {
                        stream_reads += 1;
                    }
                }
            } else {
                if pos <= new_p_old && new_p_old < end {
                    new_p = out_pos + (new_p_old - pos);
                }
                if rec_stats {
                    stream_reads += 1;
                }
                let mut bi = self.b_idx[c] as usize;
                let mut bo = self.b_off[c];
                let mut rem = w;
                while rem > 0 {
                    let blen = self.f_runs[bi].len;
                    let avail = blen - bo;
                    let take = rem.min(avail);
                    let nid = self.f_new_id[bi];
                    if let Some(last) = self.out_runs.last_mut() {
                        if last.sym == nid {
                            last.len += take;
                        } else {
                            self.out_runs.push(Run { sym: nid, len: take });
                        }
                    } else {
                        self.out_runs.push(Run { sym: nid, len: take });
                    }
                    out_pos += take;
                    rem -= take;
                    bo += take;
                    if bo == blen {
                        bi += 1;
                        bo = 0;
                    }
                }
                self.b_idx[c] = bi as u32;
                self.b_off[c] = bo;
            }
            pos = end;
        }
        debug_assert_ne!(new_p, u32::MAX);
        debug_assert!(out_pos < n);
        if rec_stats {
            st.q_indep_stream += stream_reads;
        }
        *p = new_p;
        std::mem::swap(&mut self.runs, &mut self.out_runs);
        std::mem::swap(&mut self.sym_to_rule, &mut self.new_sym_to_rule);
    }

    /// Exact 1-round monotone self-run contraction (with spiral division).
    /// Returns Some(c) if p is a fixed point (1-cycle of symbol c).
    fn do_self_run_contraction(
        &mut self,
        sigma: usize,
        _n: u32,
        p: &mut u32,
        p_offset: &mut u32,
    ) -> Option<usize> {
        let mut self_cnt = vec![0u32; sigma];
        let mut before_cnt = vec![0u32; sigma];
        for b in 0..sigma {
            let lo = self.f_start[b] as usize;
            let hi = self.f_start[b + 1] as usize;
            for e in lo..hi {
                let fr = self.f_runs[e];
                let a = fr.sym as usize;
                if a == b {
                    self_cnt[b] += fr.len;
                } else if b < a {
                    before_cnt[a] += fr.len;
                }
            }
        }

        let mut new_p_old = *p;
        let mut entry_maps: Vec<Vec<(u32, u32, u32)>> = vec![Vec::new(); sigma];
        let mut block_ks: Vec<Vec<u32>> = vec![Vec::new(); sigma];

        for c in 0..sigma {
            if self_cnt[c] == 0 {
                continue;
            }
            let lc = self.c_off[c + 1] - self.c_off[c];
            let u = before_cnt[c];
            let m = self_cnt[c];
            let v = u + m;

            // Build sorted intervals (l, r, is_d, delta)
            let mut ivs: Vec<(u32, u32, bool, i64)> = Vec::new();
            let mut off = 0u32;
            let mut t_cur = u;
            let lo_e = self.f_start[c] as usize;
            let hi_e = self.f_start[c + 1] as usize;
            for e in lo_e..hi_e {
                let fr = self.f_runs[e];
                if fr.sym as usize == c {
                    let delta = (t_cur as i64) - (off as i64);
                    ivs.push((off, off + fr.len, true, delta));
                    t_cur += fr.len;
                } else if let Some(last) = ivs.last_mut() {
                    if !last.2 && last.1 == off {
                        last.1 = off + fr.len;
                    } else {
                        ivs.push((off, off + fr.len, false, 0));
                    }
                } else {
                    ivs.push((off, off + fr.len, false, 0));
                }
                off += fr.len;
            }

            // Check p inside B_c
            if self.c_off[c] <= *p && *p < self.c_off[c + 1] {
                let mut cur_x = *p - self.c_off[c];
                let mut skip = 0u64;
                loop {
                    let idx = ivs.partition_point(|iv| iv.1 <= cur_x);
                    let (l, r, is_d, d) = ivs[idx];
                    if !is_d {
                        new_p_old = self.c_off[c] + cur_x;
                        *p_offset += (skip as u32) * self.sym_len[self.sym_to_rule[c] as usize];
                        break;
                    }
                    if d == 0 {
                        return Some(c);
                    }
                    if d > 0 {
                        let du = d as u32;
                        let q = (r - 1 - cur_x) / du + 1;
                        cur_x += q * du;
                        skip += q as u64;
                    } else {
                        let ad = (-d) as u32;
                        let q = (cur_x - l) / ad + 1;
                        cur_x -= q * ad;
                        skip += q as u64;
                    }
                }
            }

            // Forward propagate E_c = [0, u) U [v, lc) with (src_lo, cur_lo, width, k)
            let mut stk: Vec<(u32, u32, u32, u32)> = Vec::new();
            if u > 0 {
                stk.push((0, 0, u, 1));
            }
            if v < lc {
                stk.push((v, v, lc - v, 1));
            }
            let mut landed: Vec<(u32, u32, u32, u32)> = Vec::new(); // (y_lo, y_hi, k, src_lo)
            while let Some((src_lo, cur_lo, width, k)) = stk.pop() {
                let cur_hi = cur_lo + width;
                let mut idx = ivs.partition_point(|iv| iv.1 <= cur_lo);
                let mut cur = cur_lo;
                let mut s_cur = src_lo;
                while cur < cur_hi {
                    let (l, r, is_d, d) = ivs[idx];
                    let seg_hi = cur_hi.min(r);
                    let seg_w = seg_hi - cur;
                    if !is_d {
                        landed.push((cur, seg_hi, k, s_cur));
                    } else if d > 0 {
                        let du = d as u32;
                        let q_lo = (r - 1 - cur) / du + 1;
                        let q_hi = (r - 1 - (seg_hi - 1)) / du + 1;
                        if q_lo == q_hi {
                            stk.push((s_cur, cur + q_lo * du, seg_w, k + q_lo));
                        } else {
                            let split = r - (q_lo - 1) * du;
                            let w1 = split - cur;
                            stk.push((s_cur, cur + q_lo * du, w1, k + q_lo));
                            stk.push((s_cur + w1, split + q_hi * du, seg_w - w1, k + q_hi));
                        }
                    } else {
                        let ad = (-d) as u32;
                        let q_lo = (cur - l) / ad + 1;
                        let q_hi = (seg_hi - 1 - l) / ad + 1;
                        if q_lo == q_hi {
                            stk.push((s_cur, cur - q_lo * ad, seg_w, k + q_lo));
                        } else {
                            let split = l + q_lo * ad;
                            let w1 = split - cur;
                            stk.push((s_cur, cur - q_lo * ad, w1, k + q_lo));
                            stk.push((s_cur + w1, split - q_hi * ad, seg_w - w1, k + q_hi));
                        }
                    }
                    cur = seg_hi;
                    s_cur += seg_w;
                    idx += 1;
                }
            }

            landed.sort_unstable_by_key(|x| x.0);
            let mut entries: Vec<(u32, u32, u32)> = Vec::with_capacity(landed.len());
            let mut bks: Vec<u32> = Vec::new();
            let mut last_k = 0u32;
            let mut cur_bid = 0u32;
            for &(ylo, yhi, k, src_lo) in &landed {
                if bks.is_empty() || k != last_k {
                    cur_bid = bks.len() as u32;
                    bks.push(k);
                    last_k = k;
                }
                let w = yhi - ylo;
                entries.push((src_lo, src_lo + w, cur_bid));
            }
            entries.sort_unstable_by_key(|x| x.0);
            let mut merged: Vec<(u32, u32, u32)> = Vec::with_capacity(entries.len());
            for (slo, shi, bid) in entries {
                if let Some(last) = merged.last_mut() {
                    if last.2 == bid && last.1 == slo {
                        last.1 = shi;
                        continue;
                    }
                }
                merged.push((slo, shi, bid));
            }
            entry_maps[c] = merged;
            block_ks[c] = bks;
        }

        // Assign sorted new IDs
        self.new_sym_to_rule.clear();
        let mut base_id = vec![0u32; sigma];
        for c in 0..sigma {
            base_id[c] = self.new_sym_to_rule.len() as u32;
            if self_cnt[c] == 0 {
                if self.c_off[c + 1] > self.c_off[c] {
                    self.new_sym_to_rule.push(self.sym_to_rule[c]);
                }
            } else {
                let rc = self.sym_to_rule[c];
                for &k in &block_ks[c] {
                    if k == 1 {
                        self.new_sym_to_rule.push(rc);
                    } else {
                        let rid = self.rules.len() as u32;
                        self.rules.push(Rule { left: RUN_TAG | rc, right: k });
                        let len = self.sym_len[rc as usize] * k;
                        self.sym_len.push(len);
                        self.new_sym_to_rule.push(rid);
                    }
                }
            }
        }

        // Emit out_runs
        let mut e_idx = vec![0usize; sigma];
        let mut e_off = vec![0u32; sigma];
        self.out_runs.clear();
        let mut out_pos = 0u32;
        let mut new_p = u32::MAX;
        let mut pos = 0u32;
        let mut f_sym = 0usize;
        while f_sym < sigma && self.c_off[f_sym + 1] == 0 {
            f_sym += 1;
        }

        for r_idx in 0..self.runs.len() {
            let run = self.runs[r_idx];
            let c = run.sym as usize;
            let mut rem = run.len;
            let mut cur_pos = pos;
            while rem > 0 {
                while self.c_off[f_sym + 1] <= cur_pos {
                    f_sym += 1;
                }
                let avail = self.c_off[f_sym + 1] - cur_pos;
                let take = rem.min(avail);
                if c != f_sym {
                    if cur_pos <= new_p_old && new_p_old < cur_pos + take {
                        new_p = out_pos + (new_p_old - cur_pos);
                    }
                    if self_cnt[c] == 0 {
                        let nid = base_id[c];
                        if let Some(last) = self.out_runs.last_mut() {
                            if last.sym == nid {
                                last.len += take;
                            } else {
                                self.out_runs.push(Run { sym: nid, len: take });
                            }
                        } else {
                            self.out_runs.push(Run { sym: nid, len: take });
                        }
                        out_pos += take;
                    } else {
                        let emap = &entry_maps[c];
                        let mut ei = e_idx[c];
                        let mut eo = e_off[c];
                        let mut t_rem = take;
                        let b_nid = base_id[c];
                        while t_rem > 0 {
                            let (slo, shi, bid) = emap[ei];
                            let e_avail = (shi - slo) - eo;
                            let step = t_rem.min(e_avail);
                            let nid = b_nid + bid;
                            if let Some(last) = self.out_runs.last_mut() {
                                if last.sym == nid {
                                    last.len += step;
                                } else {
                                    self.out_runs.push(Run { sym: nid, len: step });
                                }
                            } else {
                                self.out_runs.push(Run { sym: nid, len: step });
                            }
                            out_pos += step;
                            t_rem -= step;
                            eo += step;
                            if eo == (shi - slo) {
                                ei += 1;
                                eo = 0;
                            }
                        }
                        e_idx[c] = ei;
                        e_off[c] = eo;
                    }
                }
                cur_pos += take;
                rem -= take;
            }
            pos += run.len;
        }
        debug_assert_ne!(new_p, u32::MAX);
        *p = new_p;
        std::mem::swap(&mut self.runs, &mut self.out_runs);
        std::mem::swap(&mut self.sym_to_rule, &mut self.new_sym_to_rule);
        None
    }
}

impl InverseBwt for BdmrKernel {
    fn invert(&mut self, ll: &[u8], _counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        // Pack into capped records and call decode_records
        let mut recs = Vec::new();
        let mut i = 0;
        while i < ll.len() {
            let c = ll[i];
            let mut j = i + 1;
            while j < ll.len() && ll[j] == c {
                j += 1;
            }
            let mut left = j - i;
            while left > 0 {
                let take = left.min(127);
                recs.push(((c as u16) << 8) | (take as u16));
                left -= take;
            }
            i = j;
        }
        self.decode_records(&recs, ll.len(), orig_ptr, out);
    }

    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(&mut self, r: &Runs, n: usize, orig_ptr: u32, out: &mut Vec<u8>, _k: Kernels) -> bool {
        self.decode_records(r.records(), n, orig_ptr, out);
        true
    }
}
