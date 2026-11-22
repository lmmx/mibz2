#![forbid(unsafe_code)]
//! Analogue B (Two-Letter Sturmian Blocks: RV = Euclid, O(log n) Grammar / Ostrowski Renormalization)
//! Stepanov Step-by-Step Solved-Analogue Transplantation Harness (Cycle 46, P2).
//!
//! Implements and verifies:
//!   1. Exact O(log n)-rule Sturmian inversion (Steps B1..B6: Euclid RV + SLP + Ostrowski/modinv shift + zero-walk emit).
//!   2. Step-by-step failure diagnostics on all 6 authenticated Wikidata blocks + non-Wikidata synthetics,
//!      attributed across all 7 working-backwards.md text classes (diagnostic audit only; never decoder input).
//!   3. MSC 37 Full Zorich-accelerated RV vs Open<16> thick-only RV (quotient histograms, active intervals, rules).
//!   4. MSC 18 Joyal-Street-Verity Traced Symmetric Monoidal Category (Piece_{Sigma*}) engine:
//!      - Exact verification of Vanishing II: Tr^{U_{(L1, 16]}}(Tr^{U_{>16}}(LF)) == Tr^{U_{>L1}}(LF).
//!      - Stratified Trace Leverage tower across L = 16 -> 8 -> 4 -> 2 -> 1.
//!      - Zero-Split Isomorphism Localization / 1D Migdal-Kadanoff Copy-Valley elimination (U_{iso, 2..16} & U_{mono, 2..16}).
//!   5. Assembled byte-exact transplant chains (Chain B-I & Chain B-II) audited on 5,000+ small BWTs,
//!      Sturmian blocks up to n=832,040, non-Wikidata synthetics, and all 6 Wikidata blocks,
//!      with 1T / 2T / 4W timing and acquisition-cost accounting vs Open<16>.

use std::collections::HashMap;
use std::fs;
use std::time::Instant;

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
// 1. Core BWT Primitives (Plain Reference Inverter + Direct BWT Encoder)
// ============================================================================

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

/// Exact bzip2 plain inverse BWT: returns (text, pos_of_row) where text[t] = L[row] for t = pos_of_row[row].
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

fn plain_invert(l: &[u8], orig_ptr: u32) -> Vec<u8> {
    plain_invert_with_pos(l, orig_ptr).0
}

/// Direct BWT of a text `t` (for small/synthetic test cases).
fn direct_bwt(t: &[u8]) -> (Vec<u8>, u32) {
    let n = t.len();
    assert!(n > 0);
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

#[derive(Clone, Copy, Debug)]
struct Run {
    start: u32,
    width: u32,
    img_start: u32, // LF(start)
    byte: u8,
}

fn extract_runs(l: &[u8]) -> (Vec<Run>, [u32; 256]) {
    let (_cnt, c) = counts_and_buckets(l);
    let mut cur = c;
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
        runs.push(Run { start, width, img_start, byte: b });
    }
    (runs, c)
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn mod_inv(a: u64, m: u64) -> u64 {
    let (mut t, mut new_t): (i128, i128) = (0, 1);
    let (mut r, mut new_r): (i128, i128) = (m as i128, a as i128);
    while new_r != 0 {
        let q = r / new_r;
        let tmp_t = t - q * new_t;
        t = new_t;
        new_t = tmp_t;
        let tmp_r = r - q * new_r;
        r = new_r;
        new_r = tmp_r;
    }
    debug_assert_eq!(r, 1);
    if t < 0 {
        t += m as i128;
    }
    t as u64
}

// ============================================================================
// 2. Analogue B: Exact Two-Letter Sturmian Solver (Stepanov Steps B1..B6)
// ============================================================================

#[derive(Clone, Copy, Debug)]
enum SlpRule {
    Byte(u8),
    Concat { left: u32, right: u32, len: u32 },
    Power { child: u32, len: u32 },
}

struct SturmianPlan {
    n: usize,
    euclid_steps: usize,
    rules: Vec<SlpRule>,
    root: u32,
    rho: usize, // exact cyclic shift from base_row to orig_ptr
}

/// Step B1: Check if L is a primitive two-letter Sturmian/Christoffel BWT: L = b^q a^p with a < b, gcd(p,q) = 1.
fn recognise_sturmian(l: &[u8]) -> Option<(u8, u8, u32, u32)> {
    let n = l.len();
    if n < 2 {
        return None;
    }
    let b = l[0];
    let mut q = 1usize;
    while q < n && l[q] == b {
        q += 1;
    }
    if q == n {
        return None;
    }
    let a = l[q];
    if a >= b {
        return None;
    }
    let mut i = q + 1;
    while i < n && l[i] == a {
        i += 1;
    }
    if i != n {
        return None;
    }
    let p = (n - q) as u32;
    let q32 = q as u32;
    if gcd_u64(p as u64, q32 as u64) != 1 {
        return None;
    }
    Some((a, b, p, q32))
}

/// Steps B1..B5: Construct the O(log n) Euclidean RV SLP and O(log n) cyclic shift rho.
fn build_sturmian_plan(l: &[u8], orig_ptr: u32) -> Option<SturmianPlan> {
    let (a, b, p, q) = recognise_sturmian(l)?;
    let n = l.len();
    let mut rules = Vec::with_capacity(64);
    rules.push(SlpRule::Byte(a)); // id 0
    rules.push(SlpRule::Byte(b)); // id 1
    let mut w0 = p;
    let mut w1 = q;
    let mut r0 = 0u32;
    let mut r1 = 1u32;
    let mut len0 = 1u32;
    let mut len1 = 1u32;
    let mut base_row = 0u32;
    let mut steps = 0usize;

    // Step B3 & B4: Zorich-accelerated Euclidean RV induction on the 2-IET
    while w0 != 1 || w1 != 1 {
        steps += 1;
        if w0 > w1 {
            let k = (w0 - 1) / w1;
            let pow_id = if k == 1 {
                r0
            } else {
                let id = rules.len() as u32;
                rules.push(SlpRule::Power { child: r0, len: len0 * k });
                id
            };
            let new_len = len0 * k + len1;
            let cat_id = rules.len() as u32;
            rules.push(SlpRule::Concat { left: pow_id, right: r1, len: new_len });
            w0 -= k * w1;
            r1 = cat_id;
            len1 = new_len;
        } else {
            let k = (w1 - 1) / w0;
            let pow_id = if k == 1 {
                r1
            } else {
                let id = rules.len() as u32;
                rules.push(SlpRule::Power { child: r1, len: len1 * k });
                id
            };
            let new_len = len1 * k + len0;
            let cat_id = rules.len() as u32;
            rules.push(SlpRule::Concat { left: pow_id, right: r0, len: new_len });
            base_row += k * w0;
            w1 -= k * w0;
            r0 = cat_id;
            len0 = new_len;
        }
    }
    let root = rules.len() as u32;
    rules.push(SlpRule::Concat { left: r0, right: r1, len: len0 + len1 });
    debug_assert_eq!((len0 + len1) as usize, n);

    // Step B5: Exact O(log n) cyclic shift rho such that Psi^rho(base_row) == orig_ptr
    let diff = ((orig_ptr as u64) + (n as u64) - (base_row as u64)) % (n as u64);
    let q_inv = mod_inv(q as u64, n as u64);
    let rho = ((diff as u128 * q_inv as u128) % (n as u128)) as usize;

    Some(SturmianPlan {
        n,
        euclid_steps: steps,
        rules,
        root,
        rho,
    })
}

fn rule_len(rules: &[SlpRule], id: u32) -> usize {
    match rules[id as usize] {
        SlpRule::Byte(_) => 1,
        SlpRule::Concat { len, .. } => len as usize,
        SlpRule::Power { len, .. } => len as usize,
    }
}

/// Step B5: O(log n) Ostrowski/SLP random-access query for byte at position t.
fn slp_query_byte(rules: &[SlpRule], mut id: u32, mut pos: usize) -> u8 {
    loop {
        match rules[id as usize] {
            SlpRule::Byte(c) => return c,
            SlpRule::Concat { left, right, .. } => {
                let ll = rule_len(rules, left);
                if pos < ll {
                    id = left;
                } else {
                    pos -= ll;
                    id = right;
                }
            }
            SlpRule::Power { child, .. } => {
                let cl = rule_len(rules, child);
                pos %= cl;
                id = child;
            }
        }
    }
}

/// Step B6: Zero-walk sequential SLP unrolling of a slice [lo, hi) of a rule into `out`.
fn slp_emit_slice(rules: &[SlpRule], id: u32, lo: usize, hi: usize, out: &mut Vec<u8>) {
    if lo >= hi {
        return;
    }
    match rules[id as usize] {
        SlpRule::Byte(c) => {
            debug_assert!(lo == 0 && hi == 1);
            out.push(c);
        }
        SlpRule::Concat { left, right, .. } => {
            let ll = rule_len(rules, left);
            if lo < ll {
                slp_emit_slice(rules, left, lo, hi.min(ll), out);
            }
            if hi > ll {
                slp_emit_slice(rules, right, lo.saturating_sub(ll), hi - ll, out);
            }
        }
        SlpRule::Power { child, .. } => {
            let cl = rule_len(rules, child);
            let first_rep = lo / cl;
            let last_rep = (hi - 1) / cl;
            if first_rep == last_rep {
                slp_emit_slice(rules, child, lo - first_rep * cl, hi - first_rep * cl, out);
                return;
            }
            // Partial head
            let head_lo = lo - first_rep * cl;
            let full_start_rep = if head_lo > 0 {
                slp_emit_slice(rules, child, head_lo, cl, out);
                first_rep + 1
            } else {
                first_rep
            };
            // Full repetitions via doubling memcpy!
            let full_reps = last_rep - full_start_rep;
            if full_reps > 0 {
                let base_start = out.len();
                slp_emit_slice(rules, child, 0, cl, out);
                let mut copied = 1usize;
                while copied < full_reps {
                    let take = copied.min(full_reps - copied);
                    let len_bytes = take * cl;
                    let src_end = base_start + len_bytes;
                    let cur_len = out.len();
                    out.resize(cur_len + len_bytes, 0);
                    out.copy_within(base_start..src_end, cur_len);
                    copied += take;
                }
            }
            // Tail repetition (may be full or partial)
            let tail_hi = hi - last_rep * cl;
            if tail_hi > 0 {
                slp_emit_slice(rules, child, 0, tail_hi, out);
            }
        }
    }
}

fn sturmian_invert(l: &[u8], orig_ptr: u32, out: &mut Vec<u8>) -> Option<SturmianPlan> {
    let plan = build_sturmian_plan(l, orig_ptr)?;
    out.clear();
    out.reserve(plan.n);
    slp_emit_slice(&plan.rules, plan.root, plan.rho, plan.n, out);
    slp_emit_slice(&plan.rules, plan.root, 0, plan.rho, out);
    Some(plan)
}

/// Generate a standard Christoffel word of length p+q with p copies of `a` and q copies of `b` (gcd(p,q)=1).
fn make_sturmian_column(p: usize, q: usize, a: u8, b: u8) -> Vec<u8> {
    assert_eq!(gcd_u64(p as u64, q as u64), 1);
    let mut l = Vec::with_capacity(p + q);
    l.resize(q, b);
    l.resize(p + q, a);
    l
}

// ============================================================================
// 3. Diagnostic Text-Class Classifier (working-backwards.md 7 classes — AUDIT ONLY)
// ============================================================================

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

/// Classify every byte of inverted text into the 7 working-backwards.md classes.
/// Used ONLY for diagnostic attribution of failure modes, NEVER as decoder input.
fn classify_text(text: &[u8]) -> Vec<TextClass> {
    let n = text.len();
    let mut cls = vec![TextClass::Structure; n];
    let mut assigned = vec![false; n];

    // Pass 1: GUIDs (optional Q/P/L/M + digits + '$' + 36-char UUID)
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

    // Pass 2: 40-hex SHA1 hashes (bounded by non-hex chars, typically quotes)
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

    // Pass 3: JSON keys ("key":)
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

    // Pass 4: Entity IDs ([QPLM][0-9]+)
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

    // Pass 5: Remaining numbers, letters (text), structure
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

// ============================================================================
// 4. MSC 18: Joyal-Street-Verity Categorical Trace Engine on Piece_{Sigma*}
//    + MSC 37: Full Zorich-Accelerated RV (same engine with U = all runs \ {mark})
// ============================================================================

#[derive(Clone, Copy, Debug)]
enum WordOp {
    Empty,
    Byte(u8),
    Concat { left: u32, right: u32, len: u32 },
    Power { child: u32, count: u32, len: u32 },
}

#[derive(Default)]
struct WordDag {
    ops: Vec<WordOp>,
}

impl WordDag {
    fn new() -> Self {
        let mut ops = Vec::with_capacity(65_536);
        ops.push(WordOp::Empty); // id 0 = empty word
        for b in 0u8..=255 {
            ops.push(WordOp::Byte(b)); // id 1 + b
        }
        Self { ops }
    }

    #[inline(always)]
    fn byte_id(b: u8) -> u32 {
        1 + b as u32
    }

    #[inline(always)]
    fn len(&self, id: u32) -> u32 {
        match self.ops[id as usize] {
            WordOp::Empty => 0,
            WordOp::Byte(_) => 1,
            WordOp::Concat { len, .. } => len,
            WordOp::Power { len, .. } => len,
        }
    }

    #[inline(always)]
    fn concat(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let len = self.len(left) + self.len(right);
        let id = self.ops.len() as u32;
        self.ops.push(WordOp::Concat { left, right, len });
        id
    }

    #[inline(always)]
    fn power(&mut self, child: u32, count: u32) -> u32 {
        if child == 0 || count == 0 {
            return 0;
        }
        if count == 1 {
            return child;
        }
        let len = self.len(child) * count;
        let id = self.ops.len() as u32;
        self.ops.push(WordOp::Power { child, count, len });
        id
    }

    fn expand(&self, id: u32, out: &mut Vec<u8>) {
        if id == 0 {
            return;
        }
        let mut stack = vec![id];
        while let Some(cur) = stack.pop() {
            match self.ops[cur as usize] {
                WordOp::Empty => {}
                WordOp::Byte(b) => out.push(b),
                WordOp::Concat { left, right, .. } => {
                    stack.push(right);
                    stack.push(left);
                }
                WordOp::Power { child, count, .. } => {
                    let start = out.len();
                    self.expand(child, out);
                    let unit_len = out.len() - start;
                    if unit_len > 0 && count > 1 {
                        let mut copied = 1u32;
                        while copied < count {
                            let take = copied.min(count - copied);
                            let bytes = take as usize * unit_len;
                            let cur_len = out.len();
                            out.resize(cur_len + bytes, 0);
                            out.copy_within(start..start + bytes, cur_len);
                            copied += take;
                        }
                    }
                }
            }
        }
    }

    fn num_rules(&self) -> usize {
        self.ops.len().saturating_sub(257)
    }
}

/// An interval-exchange morphism in Piece_{Sigma*}: a list of disjoint word-weighted strips
/// mapping [src_start, src_start+width) -> [dst_start, dst_start+width) in LF direction
/// while spelling `word` in forward (Psi) order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Band {
    src_start: u32, // in LF(X) (bottom)
    width: u32,
    dst_start: u32, // in X (top)
    word: u32,      // WordDag id (forward Psi order, through U only)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeKind {
    Active,   // appears in BOTH top (D_u) and bottom (I_u)
    Entry,    // appears ONLY in bottom (from LF(X))
    Terminal, // appears ONLY in top (into X)
}

#[derive(Clone, Copy, Debug)]
struct TraceNode {
    kind: NodeKind,
    width: u32,
    coord: u32, // Entry: src in LF(X); Terminal: dst in X; Active: unused
    word: u32,  // forward Psi word id in WordDag
    top_prev: u32,
    top_next: u32,
    bot_prev: u32,
    bot_next: u32,
}

const NIL: u32 = u32::MAX;

struct TraceStats {
    steps: usize,
    peels_q1: usize,
    peels_q2: usize,
    peels_q3: usize,
    peels_q4p: usize,
    steps_w1: usize,
    steps_w2_16: usize,
    steps_w17p: usize,
    rules_created: usize,
    max_active_nodes: usize,
    bands_out: usize,
}

/// General piece input to the JSV Categorical Trace Engine.
/// Each piece maps [top_start, top_start+width) -> [bot_start, bot_start+width)
/// (in LF direction) with forward word `word_id`. `eliminate == true` places it in U;
/// `eliminate == false` places it in X (retained).
#[derive(Clone, Copy, Debug)]
struct MorphPiece {
    top_start: u32,
    bot_start: u32,
    width: u32,
    word_id: u32,
    eliminate: bool,
}

/// Execute the Joyal-Street-Verity categorical trace Tr^U_{X, X}(f) in Piece_{Sigma*}.
/// Supports arbitrary morphisms `f: [0, n) -> [0, n)` (including composed morphisms for Vanishing II!)
/// and optional Zorich/Euclidean acceleration (peeling repeated same-partner overlaps in 1 step).
fn jsv_trace_morphism(
    n: u32,
    pieces: &[MorphPiece],
    dag: &mut WordDag,
    zorich_accel: bool,
) -> (Vec<Band>, TraceStats) {
    // Sort pieces by top_start for Top list, and by bot_start for Bottom list.
    let mut top_order: Vec<usize> = (0..pieces.len()).collect();
    top_order.sort_unstable_by_key(|&i| pieces[i].top_start);
    let mut bot_order: Vec<usize> = (0..pieces.len()).collect();
    bot_order.sort_unstable_by_key(|&i| pieces[i].bot_start);

    let mut nodes: Vec<TraceNode> = Vec::with_capacity(pieces.len() * 3 + 16);
    // Map active piece index -> node id
    let mut active_id = vec![NIL; pieces.len()];
    let mut active_count = 0usize;

    // Build Top list (coalescing adjacent retained gaps into Terminal nodes)
    let mut top_head = NIL;
    let mut top_tail = NIL;
    let mut top_pos = 0u32;
    for &idx in &top_order {
        let pc = pieces[idx];
        debug_assert_eq!(pc.top_start, top_pos);
        top_pos += pc.width;
        if pc.eliminate {
            let id = nodes.len() as u32;
            nodes.push(TraceNode {
                kind: NodeKind::Active,
                width: pc.width,
                coord: 0,
                word: pc.word_id,
                top_prev: top_tail,
                top_next: NIL,
                bot_prev: NIL,
                bot_next: NIL,
            });
            if top_tail != NIL {
                nodes[top_tail as usize].top_next = id;
            } else {
                top_head = id;
            }
            top_tail = id;
            active_id[idx] = id;
            active_count += 1;
        } else {
            // Merge with previous Terminal if contiguous
            if top_tail != NIL && nodes[top_tail as usize].kind == NodeKind::Terminal {
                nodes[top_tail as usize].width += pc.width;
            } else {
                let id = nodes.len() as u32;
                nodes.push(TraceNode {
                    kind: NodeKind::Terminal,
                    width: pc.width,
                    coord: pc.top_start,
                    word: 0,
                    top_prev: top_tail,
                    top_next: NIL,
                    bot_prev: NIL,
                    bot_next: NIL,
                });
                if top_tail != NIL {
                    nodes[top_tail as usize].top_next = id;
                } else {
                    top_head = id;
                }
                top_tail = id;
            }
        }
    }
    debug_assert_eq!(top_pos, n);

    // Build Bottom list (coalescing adjacent retained-image gaps into Entry nodes)
    let mut bot_head = NIL;
    let mut bot_tail = NIL;
    let mut bot_pos = 0u32;
    for &idx in &bot_order {
        let pc = pieces[idx];
        debug_assert_eq!(pc.bot_start, bot_pos);
        bot_pos += pc.width;
        if pc.eliminate {
            let id = active_id[idx];
            nodes[id as usize].bot_prev = bot_tail;
            nodes[id as usize].bot_next = NIL;
            if bot_tail != NIL {
                nodes[bot_tail as usize].bot_next = id;
            } else {
                bot_head = id;
            }
            bot_tail = id;
        } else {
            if bot_tail != NIL && nodes[bot_tail as usize].kind == NodeKind::Entry {
                nodes[bot_tail as usize].width += pc.width;
            } else {
                let id = nodes.len() as u32;
                nodes.push(TraceNode {
                    kind: NodeKind::Entry,
                    width: pc.width,
                    coord: pc.bot_start,
                    word: 0,
                    top_prev: NIL,
                    top_next: NIL,
                    bot_prev: bot_tail,
                    bot_next: NIL,
                });
                if bot_tail != NIL {
                    nodes[bot_tail as usize].bot_next = id;
                } else {
                    bot_head = id;
                }
                bot_tail = id;
            }
        }
    }
    debug_assert_eq!(bot_pos, n);

    let rules_before = dag.num_rules();
    let mut bands = Vec::new();
    let mut steps = 0usize;
    let mut peels_q1 = 0usize;
    let mut peels_q2 = 0usize;
    let mut peels_q3 = 0usize;
    let mut peels_q4p = 0usize;
    let mut steps_w1 = 0usize;
    let mut steps_w2_16 = 0usize;
    let mut steps_w17p = 0usize;
    let mut max_active = active_count;

    // Helpers to insert a node BEFORE `target` in Top or Bottom list
    let insert_before_top = |nodes: &mut Vec<TraceNode>, top_head: &mut u32, target: u32, new_id: u32| {
        let p = nodes[target as usize].top_prev;
        nodes[new_id as usize].top_prev = p;
        nodes[new_id as usize].top_next = target;
        nodes[target as usize].top_prev = new_id;
        if p != NIL {
            nodes[p as usize].top_next = new_id;
        } else {
            *top_head = new_id;
        }
    };
    let insert_before_bot = |nodes: &mut Vec<TraceNode>, bot_head: &mut u32, target: u32, new_id: u32| {
        let p = nodes[target as usize].bot_prev;
        nodes[new_id as usize].bot_prev = p;
        nodes[new_id as usize].bot_next = target;
        nodes[target as usize].bot_prev = new_id;
        if p != NIL {
            nodes[p as usize].bot_next = new_id;
        } else {
            *bot_head = new_id;
        }
    };
    let unlink_top = |nodes: &mut Vec<TraceNode>, top_head: &mut u32, id: u32| {
        let p = nodes[id as usize].top_prev;
        let nx = nodes[id as usize].top_next;
        if p != NIL {
            nodes[p as usize].top_next = nx;
        } else {
            *top_head = nx;
        }
        if nx != NIL {
            nodes[nx as usize].top_prev = p;
        }
    };
    let unlink_bot = |nodes: &mut Vec<TraceNode>, bot_head: &mut u32, id: u32| {
        let p = nodes[id as usize].bot_prev;
        let nx = nodes[id as usize].bot_next;
        if p != NIL {
            nodes[p as usize].bot_next = nx;
        } else {
            *bot_head = nx;
        }
        if nx != NIL {
            nodes[nx as usize].bot_prev = p;
        }
    };

    // Main left-to-right JSV Trace sweep!
    while top_head != NIL && bot_head != NIL {
        let a = top_head;
        let b = bot_head;
        let ka = nodes[a as usize].kind;
        let kb = nodes[b as usize].kind;
        let wa = nodes[a as usize].width;
        let wb = nodes[b as usize].width;

        // Track width tier of the step
        let d_min = wa.min(wb);
        if d_min == 1 {
            steps_w1 += 1;
        } else if d_min <= 16 {
            steps_w2_16 += 1;
        } else {
            steps_w17p += 1;
        }

        // Case 1: Terminal meets Entry -> emit canonical band!
        if ka == NodeKind::Terminal && kb == NodeKind::Entry {
            let w_out = dag.concat(nodes[a as usize].word, nodes[b as usize].word);
            bands.push(Band {
                src_start: nodes[b as usize].coord,
                width: d_min,
                dst_start: nodes[a as usize].coord,
                word: w_out,
            });
            nodes[a as usize].coord += d_min;
            nodes[a as usize].width -= d_min;
            nodes[b as usize].coord += d_min;
            nodes[b as usize].width -= d_min;
            if nodes[a as usize].width == 0 {
                unlink_top(&mut nodes, &mut top_head, a);
            }
            if nodes[b as usize].width == 0 {
                unlink_bot(&mut nodes, &mut bot_head, b);
            }
            steps += 1;
            peels_q1 += 1;
            continue;
        }

        // Self-cycle inside U (only possible on multi-cycle/malformed columns)
        if ka == NodeKind::Active && kb == NodeKind::Active && a == b {
            unlink_top(&mut nodes, &mut top_head, a);
            unlink_bot(&mut nodes, &mut bot_head, a);
            active_count -= 1;
            steps += 1;
            peels_q1 += 1;
            continue;
        }

        // Zorich / Euclidean spiral acceleration check:
        // Case A: a is Active, b is NOT a, and b sits immediately after a in Bottom (or a sits immediately after b in Top)!
        // Specifically:
        //   - If ka == Active && nodes[a].bot_prev == b && wa > wb:
        //     Actually, b is at bot_head, so b.bot_prev == NIL. If nodes[b].bot_next == a, then
        //     replacing the leftmost `wb` of `I_a` places the new piece RIGHT AFTER `b`, which becomes the
        //     NEW `bot_head` once `b` is unlinked! So `b` will immediately meet `a` AGAIN `q = floor((wa-1)/wb)` times!
        if zorich_accel && ka == NodeKind::Active && nodes[b as usize].bot_next == a && wa > wb {
            let q = (wa - 1) / wb;
            let consume = q * wb;
            steps += 1;
            match q {
                1 => peels_q1 += 1,
                2 => peels_q2 += 1,
                3 => peels_q3 += 1,
                _ => peels_q4p += 1,
            }
            let pow_w = dag.power(nodes[a as usize].word, q);
            let new_word = dag.concat(pow_w, nodes[b as usize].word);
            nodes[a as usize].width -= consume;
            nodes[b as usize].word = new_word;
            // b stays right at bot_head (and still precedes a in Bottom!), its D_b stays untouched if Active!
            // If kb == Entry, its coord is unchanged and its word is updated in place!
            // Wait: verify! On each peel: b enters leftmost `wb` of D_a, teleports to leftmost `wb` of I_a
            // (which is IMMEDIATELY after b in Bottom!), so removing b from bot_head and inserting at I_a's left
            // keeps b at bot_head! Repeating `q` times consumes `q*wb` from `a` and prepends `W_a^q` to `W_b`!
            continue;
        }
        // Symmetric Zorich spiral: kb == Active && nodes[a].top_next == b && wb > wa!
        if zorich_accel && kb == NodeKind::Active && nodes[a as usize].top_next == b && wb > wa {
            let q = (wb - 1) / wa;
            let consume = q * wa;
            steps += 1;
            match q {
                1 => peels_q1 += 1,
                2 => peels_q2 += 1,
                3 => peels_q3 += 1,
                _ => peels_q4p += 1,
            }
            let pow_w = dag.power(nodes[b as usize].word, q);
            let new_word = dag.concat(nodes[a as usize].word, pow_w);
            nodes[b as usize].width -= consume;
            nodes[a as usize].word = new_word;
            continue;
        }

        // Standard 1-step peel (q = 1)
        steps += 1;
        peels_q1 += 1;
        match (ka, kb) {
            // Case 2: Active(a) meets Entry(b)
            (NodeKind::Active, NodeKind::Entry) => {
                let w_new = dag.concat(nodes[a as usize].word, nodes[b as usize].word);
                let new_entry = nodes.len() as u32;
                nodes.push(TraceNode {
                    kind: NodeKind::Entry,
                    width: d_min,
                    coord: nodes[b as usize].coord,
                    word: w_new,
                    top_prev: NIL,
                    top_next: NIL,
                    bot_prev: NIL,
                    bot_next: NIL,
                });
                insert_before_bot(&mut nodes, &mut bot_head, a, new_entry);
                nodes[b as usize].coord += d_min;
                nodes[b as usize].width -= d_min;
                nodes[a as usize].width -= d_min;
                if nodes[b as usize].width == 0 {
                    unlink_bot(&mut nodes, &mut bot_head, b);
                }
                if nodes[a as usize].width == 0 {
                    unlink_top(&mut nodes, &mut top_head, a);
                    unlink_bot(&mut nodes, &mut bot_head, a);
                    active_count -= 1;
                }
            }
            // Case 3: Terminal(a) meets Active(b)
            (NodeKind::Terminal, NodeKind::Active) => {
                let w_new = dag.concat(nodes[a as usize].word, nodes[b as usize].word);
                let new_term = nodes.len() as u32;
                nodes.push(TraceNode {
                    kind: NodeKind::Terminal,
                    width: d_min,
                    coord: nodes[a as usize].coord,
                    word: w_new,
                    top_prev: NIL,
                    top_next: NIL,
                    bot_prev: NIL,
                    bot_next: NIL,
                });
                insert_before_top(&mut nodes, &mut top_head, b, new_term);
                nodes[a as usize].coord += d_min;
                nodes[a as usize].width -= d_min;
                nodes[b as usize].width -= d_min;
                if nodes[a as usize].width == 0 {
                    unlink_top(&mut nodes, &mut top_head, a);
                }
                if nodes[b as usize].width == 0 {
                    unlink_top(&mut nodes, &mut top_head, b);
                    unlink_bot(&mut nodes, &mut bot_head, b);
                    active_count -= 1;
                }
            }
            // Case 4: Active(a) meets Active(b) (a != b)
            (NodeKind::Active, NodeKind::Active) => {
                let w_new = dag.concat(nodes[a as usize].word, nodes[b as usize].word);
                let new_act = nodes.len() as u32;
                nodes.push(TraceNode {
                    kind: NodeKind::Active,
                    width: d_min,
                    coord: 0,
                    word: w_new,
                    top_prev: NIL,
                    top_next: NIL,
                    bot_prev: NIL,
                    bot_next: NIL,
                });
                insert_before_top(&mut nodes, &mut top_head, b, new_act);
                insert_before_bot(&mut nodes, &mut bot_head, a, new_act);
                active_count += 1;
                nodes[a as usize].width -= d_min;
                nodes[b as usize].width -= d_min;
                if nodes[a as usize].width == 0 {
                    unlink_top(&mut nodes, &mut top_head, a);
                    unlink_bot(&mut nodes, &mut bot_head, a);
                    active_count -= 1;
                }
                if nodes[b as usize].width == 0 {
                    unlink_top(&mut nodes, &mut top_head, b);
                    unlink_bot(&mut nodes, &mut bot_head, b);
                    active_count -= 1;
                }
                if active_count > max_active {
                    max_active = active_count;
                }
            }
            _ => unreachable!(),
        }
    }

    bands.sort_unstable_by_key(|b| b.src_start);
    let stats = TraceStats {
        steps,
        peels_q1,
        peels_q2,
        peels_q3,
        peels_q4p,
        steps_w1,
        steps_w2_16,
        steps_w17p,
        rules_created: dag.num_rules() - rules_before,
        max_active_nodes: max_active,
        bands_out: bands.len(),
    };
    (bands, stats)
}

/// Build MorphPieces for a raw BWT column `l` with an arbitrary boolean mask `elim_run[u]` over maximal runs.
/// Guarantees `orig_ptr` is ALWAYS retained (splitting its run if needed, matching Open::split).
fn build_pieces_from_runs(runs: &[Run], orig_ptr: u32, elim_run: &[bool]) -> (Vec<MorphPiece>, Vec<u32>) {
    let mut pieces = Vec::with_capacity(runs.len() + 2);
    let mut retained_rows = Vec::new();
    for (idx, r) in runs.iter().enumerate() {
        let elim = elim_run[idx];
        let end = r.start + r.width;
        if elim && orig_ptr >= r.start && orig_ptr < end {
            if r.start < orig_ptr {
                let lw = orig_ptr - r.start;
                pieces.push(MorphPiece {
                    top_start: r.start,
                    bot_start: r.img_start,
                    width: lw,
                    word_id: WordDag::byte_id(r.byte),
                    eliminate: true,
                });
            }
            pieces.push(MorphPiece {
                top_start: orig_ptr,
                bot_start: r.img_start + (orig_ptr - r.start),
                width: 1,
                word_id: WordDag::byte_id(r.byte),
                eliminate: false,
            });
            retained_rows.push(orig_ptr);
            if orig_ptr + 1 < end {
                let rw = end - (orig_ptr + 1);
                pieces.push(MorphPiece {
                    top_start: orig_ptr + 1,
                    bot_start: r.img_start + (orig_ptr + 1 - r.start),
                    width: rw,
                    word_id: WordDag::byte_id(r.byte),
                    eliminate: true,
                });
            }
        } else {
            pieces.push(MorphPiece {
                top_start: r.start,
                bot_start: r.img_start,
                width: r.width,
                word_id: WordDag::byte_id(r.byte),
                eliminate: elim,
            });
            if !elim {
                for row in r.start..end {
                    retained_rows.push(row);
                }
            }
        }
    }
    (pieces, retained_rows)
}

/// Wire and emit from any JSV-traced band table + retained rows, returning the exact inverted text
/// and dictionary statistics (distinct words, spelling bytes, % inline <= 15B).
struct EmitReport {
    out: Vec<u8>,
    retained_m: usize,
    distinct_words: usize,
    spelling_bytes: usize,
    pct_inline15: f64,
}

fn wire_and_emit_from_bands(
    l: &[u8],
    runs: &[Run],
    orig_ptr: u32,
    elim_run: &[bool],
    bands: &[Band],
    dag: &WordDag,
) -> EmitReport {
    let n = l.len();
    // Map each retained row x -> rank in 0..m
    let mut rank_of_row = vec![u32::MAX; n];
    let mut row_of_rank: Vec<u32> = Vec::new();
    let mut lf_of_rank: Vec<u32> = Vec::new();
    let mut byte_of_rank: Vec<u8> = Vec::new();

    for (idx, r) in runs.iter().enumerate() {
        let elim = elim_run[idx];
        let end = r.start + r.width;
        if elim {
            if orig_ptr >= r.start && orig_ptr < end {
                let rk = row_of_rank.len() as u32;
                rank_of_row[orig_ptr as usize] = rk;
                row_of_rank.push(orig_ptr);
                lf_of_rank.push(r.img_start + (orig_ptr - r.start));
                byte_of_rank.push(r.byte);
            }
        } else {
            for off in 0..r.width {
                let row = r.start + off;
                let rk = row_of_rank.len() as u32;
                rank_of_row[row as usize] = rk;
                row_of_rank.push(row);
                lf_of_rank.push(r.img_start + off);
                byte_of_rank.push(r.byte);
            }
        }
    }
    let m = row_of_rank.len();
    let mark_rk = rank_of_row[orig_ptr as usize] as usize;

    // Deduplicate band words (epi-mono image factorization in Piece_{Sigma*}!)
    let mut dict_map: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut dict_words: Vec<Vec<u8>> = Vec::new();
    let mut band_word_code: Vec<u32> = Vec::with_capacity(bands.len());
    let mut tmp = Vec::new();
    let mut inline15_bands = 0usize;
    for b in bands {
        tmp.clear();
        dag.expand(b.word, &mut tmp);
        if tmp.len() <= 15 {
            inline15_bands += 1;
        }
        let code = if let Some(&c) = dict_map.get(&tmp) {
            c
        } else {
            let c = dict_words.len() as u32;
            dict_map.insert(tmp.clone(), c);
            dict_words.push(tmp.clone());
            c
        };
        band_word_code.push(code);
    }

    // Build forward retained permutation:
    // For retained row x (rank rx), y = LF(x) falls in band bi => next_x_bwd = dst_start + (y - src_start).
    // In forward (Psi) order: next_fwd[rank(next_x_bwd)] = rx, emitting dict_words[band_code] + [L[x]]!
    let mut next_fwd = vec![0u32; m];
    let mut word_code_fwd = vec![0u32; m];
    let mut tail_byte_fwd = vec![0u8; m];

    for rx in 0..m {
        let y = lf_of_rank[rx];
        // Binary search band containing y
        let bi = bands
            .binary_search_by(|b| {
                if y < b.src_start {
                    std::cmp::Ordering::Greater
                } else if y >= b.src_start + b.width {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .expect("LF(retained) must lie in an exact band");
        let b = &bands[bi];
        let dst_row = b.dst_start + (y - b.src_start);
        let rdst = rank_of_row[dst_row as usize] as usize;
        debug_assert_ne!(rdst, u32::MAX as usize);
        next_fwd[rdst] = rx as u32;
        word_code_fwd[rdst] = band_word_code[bi];
        tail_byte_fwd[rdst] = byte_of_rank[rx];
    }

    // Emit forward from mark_rk!
    let mut out = Vec::with_capacity(n);
    let mut cur = mark_rk;
    for _ in 0..m {
        let w = &dict_words[word_code_fwd[cur] as usize];
        out.extend_from_slice(w);
        out.push(tail_byte_fwd[cur]);
        cur = next_fwd[cur] as usize;
        if cur == mark_rk || out.len() >= n {
            break;
        }
    }
    if out.len() < n && !out.is_empty() {
        let period = out.len();
        while out.len() < n {
            let take = period.min(n - out.len());
            let cur_len = out.len();
            out.resize(cur_len + take, 0);
            out.copy_within(0..take, cur_len);
        }
    }
    out.truncate(n);

    let spelling_bytes: usize = dict_words.iter().map(|w| w.len()).sum();
    let pct_inline15 = if bands.is_empty() {
        100.0
    } else {
        100.0 * inline15_bands as f64 / bands.len() as f64
    };

    EmitReport {
        out,
        retained_m: m,
        distinct_words: dict_words.len(),
        spelling_bytes,
        pct_inline15,
    }
}

/// Verify JSV Vanishing II: Tr^{U_{(L1, L2]}}(Tr^{U_{>L2}}(LF)) == Tr^{U_{>L1}}(LF).
/// Composes the first-stage trace bands (on X_{L2}) into a morphism on X_{L2} indexed 0..m2,
/// marks X_{L2} \ X_{L1} as eliminated, runs the EXACT SAME JSV trace engine, and compares
/// every resulting (retained_row -> next_retained_row, spelled_word) pair against one-shot Tr^{U_{>L1}}!
fn verify_vanishing_ii(
    l: &[u8],
    runs: &[Run],
    orig_ptr: u32,
    l1: u32,
    l2: u32,
) -> (bool, usize, usize, usize, usize) {
    // Stage 1: one-shot Tr^{U_{>L2}}(LF)
    let elim2: Vec<bool> = runs.iter().map(|r| r.width > l2).collect();
    let (pieces2, ret2) = build_pieces_from_runs(runs, orig_ptr, &elim2);
    let mut dag2 = WordDag::new();
    let (bands2, st2) = jsv_trace_morphism(l.len() as u32, &pieces2, &mut dag2, false);

    // Build exact induced morphism on X_{L2} (compressed coordinates 0..m2)
    let m2 = ret2.len() as u32;
    let mut rank2 = vec![u32::MAX; l.len()];
    for (i, &row) in ret2.iter().enumerate() {
        rank2[row as usize] = i as u32;
    }

    // Build LF-image sorted pieces of X_{L2} -> X_{L2}:
    // Each retained run r (width <= L2, or singleton orig_ptr) maps by LF to [img_start, img_start + w),
    // which is partitioned by `bands2` into sub-strips mapping into `X_{L2}`!
    let mut stage2_pieces: Vec<MorphPiece> = Vec::new();
    for (idx, r) in runs.iter().enumerate() {
        let mut handle_seg = |seg_start: u32, seg_img: u32, seg_w: u32, is_mark_only: bool| {
            let mut off = 0u32;
            while off < seg_w {
                let y = seg_img + off;
                let bi = bands2
                    .binary_search_by(|b| {
                        if y < b.src_start {
                            std::cmp::Ordering::Greater
                        } else if y >= b.src_start + b.width {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    })
                    .expect("band must cover LF(X2)");
                let b = &bands2[bi];
                let take = (seg_w - off).min(b.src_start + b.width - y);
                let dst_row = b.dst_start + (y - b.src_start);
                let top_r = rank2[(seg_start + off) as usize];
                let bot_r = rank2[dst_row as usize];
                // Full forward word for this X2 step: band_word + [L[x]]
                let full_w = dag2.concat(b.word, WordDag::byte_id(r.byte));
                let elim_in_stage2 = !is_mark_only && r.width > l1;
                stage2_pieces.push(MorphPiece {
                    top_start: top_r,
                    bot_start: bot_r,
                    width: take,
                    word_id: full_w,
                    eliminate: elim_in_stage2,
                });
                off += take;
            }
        };

        if elim2[idx] {
            if orig_ptr >= r.start && orig_ptr < r.start + r.width {
                handle_seg(orig_ptr, r.img_start + (orig_ptr - r.start), 1, true);
            }
        } else if r.width > l1 && orig_ptr >= r.start && orig_ptr < r.start + r.width {
            if r.start < orig_ptr {
                handle_seg(r.start, r.img_start, orig_ptr - r.start, false);
            }
            handle_seg(orig_ptr, r.img_start + (orig_ptr - r.start), 1, true);
            if orig_ptr + 1 < r.start + r.width {
                handle_seg(
                    orig_ptr + 1,
                    r.img_start + (orig_ptr + 1 - r.start),
                    r.start + r.width - (orig_ptr + 1),
                    false,
                );
            }
        } else {
            handle_seg(r.start, r.img_start, r.width, false);
        }
    }

    // Stage 2: Trace out U_{(L1, L2]} on the already-traced morphism!
    let (bands_2stage, st_stage2) = jsv_trace_morphism(m2, &stage2_pieces, &mut dag2, false);

    // One-shot reference: Tr^{U_{>L1}}(LF)
    let elim1: Vec<bool> = runs.iter().map(|r| r.width > l1).collect();
    let (pieces1, ret1) = build_pieces_from_runs(runs, orig_ptr, &elim1);
    let mut dag1 = WordDag::new();
    let (bands_1shot, st1) = jsv_trace_morphism(l.len() as u32, &pieces1, &mut dag1, false);

    // Verify exact row-by-row morphism equality on EVERY retained row x in X_{L1}!
    // For each x in X_{L1}:
    //   1-shot: y = LF(x) -> band1(y) gives (dst1, word1 + [L[x]])
    //   2-stage: x has rank2[x] in X_{L2}; in stage2_pieces, rank2[x] maps to some bot_r2 with word_a,
    //            then bands_2stage(bot_r2) gives (dst_r2, word_b) => dst2 = ret2[dst_r2], word2 = word_b + word_a!
    let mut stage2_by_top = stage2_pieces.clone();
    stage2_by_top.sort_unstable_by_key(|p| p.top_start);

    // Build LF lookup for X1
    let mut lf_of_row = vec![0u32; l.len()];
    let mut byte_of_row = vec![0u8; l.len()];
    for r in runs {
        for off in 0..r.width {
            lf_of_row[(r.start + off) as usize] = r.img_start + off;
            byte_of_row[(r.start + off) as usize] = r.byte;
        }
    }

    let mut exact = true;
    let mut w1_buf = Vec::new();
    let mut w2_buf = Vec::new();
    for &x in &ret1 {
        // 1-shot image
        let y = lf_of_row[x as usize];
        let b1i = bands_1shot
            .binary_search_by(|b| {
                if y < b.src_start {
                    std::cmp::Ordering::Greater
                } else if y >= b.src_start + b.width {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .unwrap();
        let b1 = &bands_1shot[b1i];
        let dst1 = b1.dst_start + (y - b1.src_start);
        w1_buf.clear();
        dag1.expand(b1.word, &mut w1_buf);
        w1_buf.push(byte_of_row[x as usize]);

        // 2-stage image
        let r2 = rank2[x as usize];
        let pi = stage2_by_top
            .binary_search_by(|p| {
                if r2 < p.top_start {
                    std::cmp::Ordering::Greater
                } else if r2 >= p.top_start + p.width {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .unwrap();
        let pc = &stage2_by_top[pi];
        let y2 = pc.bot_start + (r2 - pc.top_start);
        let b2i = bands_2stage
            .binary_search_by(|b| {
                if y2 < b.src_start {
                    std::cmp::Ordering::Greater
                } else if y2 >= b.src_start + b.width {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .unwrap();
        let b2 = &bands_2stage[b2i];
        let dst2_rank = b2.dst_start + (y2 - b2.src_start);
        let dst2 = ret2[dst2_rank as usize];
        w2_buf.clear();
        dag2.expand(b2.word, &mut w2_buf);
        dag2.expand(pc.word_id, &mut w2_buf);

        if dst1 != dst2 || w1_buf != w2_buf {
            exact = false;
            break;
        }
    }

    (exact, ret1.len(), st1.steps, st2.steps + st_stage2.steps, bands_1shot.len())
}

/// Identify thin runs in [2, limit] that are:
///  - `iso`: exact-width isomorphisms LF(U_u) == U_v (w_u == w_v, zero split in both directions!)
///  - `mono`: unsplit forward monomorphisms LF(U_u) subseteq U_v (zero forward split!)
/// Purely from decoder-available `runs`!
fn detect_copy_valleys(runs: &[Run], limit: u32) -> (Vec<bool>, Vec<bool>) {
    let r = runs.len();
    let mut iso = vec![false; r];
    let mut mono = vec![false; r];
    // `runs` is already sorted by `start`!
    for (u, run) in runs.iter().enumerate() {
        if run.width < 2 || run.width > limit {
            continue;
        }
        let img_s = run.img_start;
        let img_e = img_s + run.width;
        // Binary search target run v containing img_s
        let v = runs.partition_point(|x| x.start + x.width <= img_s);
        if v < r && runs[v].start <= img_s && img_e <= runs[v].start + runs[v].width {
            mono[u] = true;
            if runs[v].start == img_s && runs[v].width == run.width {
                iso[u] = true;
            }
        }
    }
    (iso, mono)
}

// ============================================================================
// 5. Production-Style Assembled Chains for Exact Timing (1T / 2T / 4W)
//    Comparing Open16-Serial, Chain B-I (Sturmian + JSV<16> + XOR-BiEmit),
//    and Chain B-II (Sturmian + JSV<16> + MK1D Copy-Valley Decimation + XOR-BiEmit)
// ============================================================================

struct BenchChainMetrics {
    construct_us: f64,
    emit_us: f64,
    total_us: f64,
}

/// Compact inline-16 dictionary + XOR-BiEmit / Serial-Emit kernel for fair apples-to-apples timing.
struct FastKernel {
    use_xor_biemit: bool,
    use_sturmian_fastpath: bool,
    use_mk1d_valleys: bool,
}

impl FastKernel {
    fn invert_timed(&self, l: &[u8], orig_ptr: u32, out: &mut Vec<u8>) -> BenchChainMetrics {
        let t0 = Instant::now();
        // Step B1 fast-path check: O(1)-check if r == 2 Sturmian!
        if self.use_sturmian_fastpath {
            if let Some(plan) = build_sturmian_plan(l, orig_ptr) {
                let t1 = Instant::now();
                out.clear();
                out.reserve(plan.n);
                slp_emit_slice(&plan.rules, plan.root, plan.rho, plan.n, out);
                slp_emit_slice(&plan.rules, plan.root, 0, plan.rho, out);
                let t2 = Instant::now();
                return BenchChainMetrics {
                    construct_us: (t1 - t0).as_secs_f64() * 1e6,
                    emit_us: (t2 - t1).as_secs_f64() * 1e6,
                    total_us: (t2 - t0).as_secs_f64() * 1e6,
                };
            }
        }

        let n = l.len();
        let limit = 16u32;

        // S1: Single-pass run scan + thick strip collection + retained queue fill
        let (runs, _c) = extract_runs(l);
        let mut elim_run: Vec<bool> = runs.iter().map(|r| r.width > limit).collect();

        // Optional Chain B-II: MK1D Copy-Valley / Categorical Isomorphism Detection (2 <= w <= 16)
        if self.use_mk1d_valleys {
            let (iso, _mono) = detect_copy_valleys(&runs, limit);
            for i in 0..runs.len() {
                if iso[i] {
                    elim_run[i] = true;
                }
            }
        }

        let (pieces, _ret_rows) = build_pieces_from_runs(&runs, orig_ptr, &elim_run);
        let mut dag = WordDag::new();
        let (bands, _st) = jsv_trace_morphism(n as u32, &pieces, &mut dag, false);

        // S3: Compact inline-16 + overflow spelling dictionary
        let mut inline16: Vec<[u8; 16]> = Vec::with_capacity(bands.len() + 256);
        let mut lens: Vec<u8> = Vec::with_capacity(bands.len() + 256);
        let mut long_words: Vec<Vec<u8>> = Vec::new();
        let mut band_code: Vec<u32> = Vec::with_capacity(bands.len());
        let mut tmp = Vec::with_capacity(64);
        let mut dedup: HashMap<Vec<u8>, u32> = HashMap::with_capacity(bands.len().min(8192));

        // Note: each band stores only the U-word; retained byte is appended per step (or fused into 16B!)
        for b in &bands {
            tmp.clear();
            dag.expand(b.word, &mut tmp);
            let code = if let Some(&c) = dedup.get(&tmp) {
                c
            } else {
                let c = inline16.len() as u32;
                let mut slot = [0u8; 16];
                if tmp.len() <= 15 {
                    slot[..tmp.len()].copy_from_slice(&tmp);
                    inline16.push(slot);
                    lens.push(tmp.len() as u8);
                } else {
                    let lid = long_words.len() as u32;
                    long_words.push(tmp.clone());
                    slot[0..4].copy_from_slice(&lid.to_le_bytes());
                    inline16.push(slot);
                    lens.push(255);
                }
                dedup.insert(tmp.clone(), c);
                c
            };
            band_code.push(code);
        }

        // S4: Build retained row rank map & edges
        let mut rank_of_row = vec![u32::MAX; n];
        let mut lf_of_rank: Vec<u32> = Vec::new();
        let mut byte_of_rank: Vec<u8> = Vec::new();
        for (idx, r) in runs.iter().enumerate() {
            let end = r.start + r.width;
            if elim_run[idx] {
                if orig_ptr >= r.start && orig_ptr < end {
                    let rk = lf_of_rank.len() as u32;
                    rank_of_row[orig_ptr as usize] = rk;
                    lf_of_rank.push(r.img_start + (orig_ptr - r.start));
                    byte_of_rank.push(r.byte);
                }
            } else {
                for off in 0..r.width {
                    let row = r.start + off;
                    let rk = lf_of_rank.len() as u32;
                    rank_of_row[row as usize] = rk;
                    lf_of_rank.push(r.img_start + off);
                    byte_of_rank.push(r.byte);
                }
            }
        }
        let m = lf_of_rank.len();
        let mark = rank_of_row[orig_ptr as usize] as usize;

        // Pack (next_or_xor, code, byte) cleanly for emit
        let mut edges_fwd = vec![0u32; m];
        let mut codes_fwd = vec![0u32; m];
        let mut tail_fwd = vec![0u8; m];

        // Two-pointer sweep over byte-sorted LF(X) vs sorted bands would be O(m + B);
        // here we use direct binary search per retained row (keeping construct fair across both chains).
        let use_bi = self.use_xor_biemit && m >= 65_536;
        let mut xor_edges = if use_bi { vec![0u32; m] } else { Vec::new() };
        let mut pred_mark = 0usize;

        for rx in 0..m {
            let y = lf_of_rank[rx];
            let bi = bands
                .binary_search_by(|b| {
                    if y < b.src_start {
                        std::cmp::Ordering::Greater
                    } else if y >= b.src_start + b.width {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
                .unwrap();
            let b = &bands[bi];
            let dst_row = b.dst_start + (y - b.src_start);
            let rdst = rank_of_row[dst_row as usize] as usize;
            if use_bi {
                xor_edges[rdst] ^= rx as u32;
                xor_edges[rx] ^= rdst as u32;
                if rx == mark {
                    pred_mark = rdst;
                }
            } else {
                edges_fwd[rdst] = rx as u32;
            }
            codes_fwd[rdst] = band_code[bi];
            tail_fwd[rdst] = byte_of_rank[rx];
        }

        let t1 = Instant::now();

        // S5: Emit
        out.clear();
        if use_bi {
            // Check single cycle or handle short cycle cleanly via bidirectional XOR walk!
            out.resize(n + 16, 0);
            let mut f_cur = mark;
            let mut f_prev = pred_mark;
            let mut b_cur = pred_mark;
            let mut b_prev = mark;
            let mut lo = 0usize;
            let mut hi = n;
            let mut visited = 0usize;
            while lo < hi && visited < m {
                // Forward step from f_cur
                let fc = codes_fwd[f_cur] as usize;
                let fl = lens[fc];
                if fl != 255 {
                    let flen = fl as usize;
                    if lo + 16 <= hi {
                        out[lo..lo + 16].copy_from_slice(&inline16[fc]);
                        lo += flen;
                    } else {
                        let take = flen.min(hi - lo);
                        out[lo..lo + take].copy_from_slice(&inline16[fc][..take]);
                        lo += take;
                    }
                } else {
                    let lid = u32::from_le_bytes([
                        inline16[fc][0], inline16[fc][1], inline16[fc][2], inline16[fc][3],
                    ]) as usize;
                    let lw = &long_words[lid];
                    let take = lw.len().min(hi - lo);
                    out[lo..lo + take].copy_from_slice(&lw[..take]);
                    lo += take;
                }
                if lo < hi {
                    out[lo] = tail_fwd[f_cur];
                    lo += 1;
                }
                visited += 1;
                if lo >= hi || visited >= m {
                    break;
                }
                let f_next = (xor_edges[f_cur] ^ (f_prev as u32)) as usize;
                f_prev = f_cur;
                f_cur = f_next;

                // Backward step at b_cur (emits tail_fwd[b_cur] at the end, preceded by word[b_cur])
                if hi > lo {
                    hi -= 1;
                    out[hi] = tail_fwd[b_cur];
                }
                let bc = codes_fwd[b_cur] as usize;
                let bl = lens[bc];
                if bl != 255 {
                    let blen = bl as usize;
                    let take = blen.min(hi - lo);
                    hi -= take;
                    out[hi..hi + take].copy_from_slice(&inline16[bc][blen - take..blen]);
                } else {
                    let lid = u32::from_le_bytes([
                        inline16[bc][0], inline16[bc][1], inline16[bc][2], inline16[bc][3],
                    ]) as usize;
                    let lw = &long_words[lid];
                    let take = lw.len().min(hi - lo);
                    hi -= take;
                    out[hi..hi + take].copy_from_slice(&lw[lw.len() - take..]);
                }
                visited += 1;
                if lo >= hi || visited >= m {
                    break;
                }
                let b_next = (xor_edges[b_cur] ^ (b_prev as u32)) as usize;
                b_prev = b_cur;
                b_cur = b_next;
                if f_cur == mark {
                    break;
                }
            }
            out.truncate(n);
            if visited < m && lo < hi {
                // Periodic short cycle fallback: repeat prefix
                let mut serial_out = Vec::with_capacity(n);
                let mut cur = mark;
                let mut prev = pred_mark;
                for _ in 0..m {
                    let c = codes_fwd[cur] as usize;
                    let ln = lens[c];
                    if ln != 255 {
                        serial_out.extend_from_slice(&inline16[c][..ln as usize]);
                    } else {
                        let lid = u32::from_le_bytes([
                            inline16[c][0], inline16[c][1], inline16[c][2], inline16[c][3],
                        ]) as usize;
                        serial_out.extend_from_slice(&long_words[lid]);
                    }
                    serial_out.push(tail_fwd[cur]);
                    let nx = (xor_edges[cur] ^ prev as u32) as usize;
                    prev = cur;
                    cur = nx;
                    if cur == mark || serial_out.len() >= n {
                        break;
                    }
                }
                let period = serial_out.len();
                while serial_out.len() < n {
                    let take = period.min(n - serial_out.len());
                    let cl = serial_out.len();
                    serial_out.resize(cl + take, 0);
                    serial_out.copy_within(0..take, cl);
                }
                serial_out.truncate(n);
                *out = serial_out;
            }
        } else {
            out.resize(n + 16, 0);
            let mut pos = 0usize;
            let mut cur = mark;
            for _ in 0..m {
                let c = codes_fwd[cur] as usize;
                let ln = lens[c];
                if ln != 255 {
                    let l_us = ln as usize;
                    out[pos..pos + 16].copy_from_slice(&inline16[c]);
                    pos += l_us;
                } else {
                    let lid = u32::from_le_bytes([
                        inline16[c][0], inline16[c][1], inline16[c][2], inline16[c][3],
                    ]) as usize;
                    let lw = &long_words[lid];
                    let take = lw.len().min(n - pos);
                    out[pos..pos + take].copy_from_slice(&lw[..take]);
                    pos += take;
                }
                if pos < n {
                    out[pos] = tail_fwd[cur];
                    pos += 1;
                }
                cur = edges_fwd[cur] as usize;
                if cur == mark || pos >= n {
                    break;
                }
            }
            if pos < n && pos > 0 {
                let period = pos;
                while pos < n {
                    let take = period.min(n - pos);
                    out.copy_within(0..take, pos);
                    pos += take;
                }
            }
            out.truncate(n);
        }

        let t2 = Instant::now();
        BenchChainMetrics {
            construct_us: (t1 - t0).as_secs_f64() * 1e6,
            emit_us: (t2 - t1).as_secs_f64() * 1e6,
            total_us: (t2 - t0).as_secs_f64() * 1e6,
        }
    }
}

// ============================================================================
// 6. Synthetic Non-Wikidata Generators (Deterministic PRNG)
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

/// Non-Wikidata synthetic API JSON log (unique request UUIDs + repeated artifact SHA1s + JSON keys).
fn make_synthetic_json_log(num_records: usize) -> Vec<u8> {
    let mut rng = XorShift64(0x9E37_79B9_7F4A_7C15);
    // Pool of 60 repeated SHA1s (each repeated ~8 times on average for 480 records!)
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

/// Non-Wikidata synthetic dpkg/syslog style structured text (no UUIDs/SHA1s).
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

// ============================================================================
// 7. Main Verification, Diagnostics, & Benchmarking Driver
// ============================================================================

fn main() {
    println!("========================================================================");
    println!("ANALOGUE B (TWO-LETTER STURMIAN BLOCKS) — STEPANOV TRANSPLANTATION PROBE");
    println!("========================================================================\n");

    // ------------------------------------------------------------------------
    // PART 1: Verify Exact Sturmian Steps B1..B6 on Fibonacci & Christoffel BWTs
    // ------------------------------------------------------------------------
    println!("--- PART 1: Exactness of Analogue B (Steps B1..B6) on Sturmian Words ---");
    let mut sturmian_checks = 0usize;
    let mut out = Vec::new();
    // All coprime (p, q) with p+q <= 35, testing EVERY orig_ptr!
    for n in 2..=30usize {
        for p in 1..n {
            let q = n - p;
            if gcd_u64(p as u64, q as u64) != 1 {
                continue;
            }
            let l = make_sturmian_column(p, q, b'0', b'1');
            for orig in 0..n as u32 {
                let ref_text = plain_invert(&l, orig);
                let plan = sturmian_invert(&l, orig, &mut out).expect("must recognise Sturmian");
                assert_eq!(out, ref_text, "Sturmian mismatch at p={p}, q={q}, orig={orig}");
                // Also verify Step B5 O(log n) Ostrowski random access at every position!
                for t in 0..n {
                    let b_slp = slp_query_byte(&plan.rules, plan.root, (plan.rho + t) % n);
                    assert_eq!(b_slp, ref_text[t], "Step B5 Ostrowski query mismatch");
                }
                sturmian_checks += 1;
            }
        }
    }

    // Large Fibonacci Sturmian words up to F_29 = 832,040 (Wikidata-block scale!)
    let mut fib_a = 1usize;
    let mut fib_b = 2usize;
    while fib_a + fib_b <= 900_000 {
        let p = fib_b;
        let q = fib_a;
        let n = p + q;
        let l = make_sturmian_column(p, q, b'a', b'b');
        let orig = ((n * 37) / 100) as u32;
        let t_p0 = Instant::now();
        let ref_text = plain_invert(&l, orig);
        let t_plain_us = t_p0.elapsed().as_secs_f64() * 1e6;

        let t_s0 = Instant::now();
        let plan = sturmian_invert(&l, orig, &mut out).expect("Fibonacci must be Sturmian");
        let t_sturm_us = t_s0.elapsed().as_secs_f64() * 1e6;

        assert_eq!(out, ref_text, "Large Fibonacci mismatch at n={n}");
        // Verify Step B5 Ostrowski O(log n) random access at 64 strided positions
        for k in 0..64 {
            let pos = (k * 13_337) % n;
            let b_slp = slp_query_byte(&plan.rules, plan.root, (plan.rho + pos) % n);
            assert_eq!(b_slp, ref_text[pos]);
        }
        sturmian_checks += 1;
        if n >= 10_000 {
            println!(
                "  Fib Sturmian n={:7} (p={:6}, q={:6}): Euclid steps={:2}, SLP rules={:2}, m=0, Sturmian={:7.1} us vs Plain={:8.1} us ({:5.1}x faster) [EXACT]",
                n, p, q, plan.euclid_steps, plan.rules.len(), t_sturm_us, t_plain_us, t_plain_us / t_sturm_us.max(0.1)
            );
        }
        let next = fib_a + fib_b;
        fib_a = fib_b;
        fib_b = next;
    }
    println!("  Verified {} Sturmian/Christoffel inversions + Ostrowski queries 100% EXACT.\n", sturmian_checks);

    // ------------------------------------------------------------------------
    // PART 2: 5,000+ Small-Case Exhaustive Exactness Audit of All Substitutes & Chains
    // ------------------------------------------------------------------------
    println!("--- PART 2: 5,000+ Small-Case Exactness Audit (JSV Trace, Vanishing II, Zorich RV, Chains B-I/B-II) ---");
    let mut small_cases = 0usize;
    let mut vanishing_ii_checks = 0usize;
    let kernel_bi = FastKernel { use_xor_biemit: true, use_sturmian_fastpath: true, use_mk1d_valleys: false };
    let kernel_bii = FastKernel { use_xor_biemit: true, use_sturmian_fastpath: true, use_mk1d_valleys: true };
    let kernel_ser = FastKernel { use_xor_biemit: false, use_sturmian_fastpath: false, use_mk1d_valleys: false };

    let mut audit_one = |t: &[u8], check_vanishing: bool| {
        let (l, orig) = direct_bwt(t);
        let ref_text = plain_invert(&l, orig);
        assert_eq!(ref_text, t, "BWT roundtrip sanity");

        let (runs, _) = extract_runs(&l);

        // Full Zorich-accelerated RV (eliminate ALL runs except mark -> m = 1!)
        let elim_all = vec![true; runs.len()];
        let (pieces_all, _) = build_pieces_from_runs(&runs, orig, &elim_all);
        let mut dag_all = WordDag::new();
        let (bands_all, _) = jsv_trace_morphism(l.len() as u32, &pieces_all, &mut dag_all, true);
        let rep_all = wire_and_emit_from_bands(&l, &runs, orig, &elim_all, &bands_all, &dag_all);
        assert_eq!(rep_all.out, ref_text, "Full Zorich RV mismatch on len={}", t.len());

        // Chain B-I, Chain B-II, Open-Serial
        let mut o1 = Vec::new();
        let mut o2 = Vec::new();
        let mut o3 = Vec::new();
        kernel_ser.invert_timed(&l, orig, &mut o1);
        kernel_bi.invert_timed(&l, orig, &mut o2);
        kernel_bii.invert_timed(&l, orig, &mut o3);
        assert_eq!(o1, ref_text, "Open-Serial mismatch");
        assert_eq!(o2, ref_text, "Chain B-I mismatch");
        assert_eq!(o3, ref_text, "Chain B-II mismatch");

        if check_vanishing {
            let (ok, _, _, _, _) = verify_vanishing_ii(&l, &runs, orig, 1, 3);
            assert!(ok, "Vanishing II (L1=1, L2=3) failed!");
            vanishing_ii_checks += 1;
        }
        small_cases += 1;
    };

    // Exhaustive binary strings of lengths 2..=12 (4,092 + 2 = 4,094 strings)
    for len in 2..=12usize {
        for mask in 0..(1usize << len) {
            let t: Vec<u8> = (0..len).map(|b| b'0' + ((mask >> b) & 1) as u8).collect();
            audit_one(&t, len <= 9);
        }
    }
    // Exhaustive ternary strings of lengths 2..=6 (3^2 + ... + 3^6 = 1,089 strings)
    for len in 2..=6usize {
        let total = 3usize.pow(len as u32);
        for mut code in 0..total {
            let mut t = Vec::with_capacity(len);
            for _ in 0..len {
                t.push(b'a' + (code % 3) as u8);
                code /= 3;
            }
            audit_one(&t, len <= 5);
        }
    }
    // Random 4-ary, 16-ary, 64-ary + periodic strings (500 cases, lengths 20..120)
    let mut rng = XorShift64(0xCAFE_BABE_1234_5678);
    for case in 0..500usize {
        let len = 20 + (rng.next() as usize % 100);
        let alpha = match case % 3 {
            0 => 4u8,
            1 => 16u8,
            _ => 64u8,
        };
        let t: Vec<u8> = (0..len).map(|_| b' ' + (rng.next() as u8 % alpha)).collect();
        audit_one(&t, case < 100);
    }
    println!(
        "  Passed {small_cases} small BWT cases (exhaustive binary n<=12, ternary n<=6, random 4/16/64-ary) + {vanishing_ii_checks} Vanishing-II checks 100% BYTE-EXACT.\n"
    );

    // ------------------------------------------------------------------------
    // PART 3: Load 6 Authenticated Wikidata Blocks + 2 Non-Wikidata Synthetics
    // ------------------------------------------------------------------------
    struct LoadedBlock {
        name: String,
        l: Vec<u8>,
        orig_ptr: u32,
        is_wikidata: bool,
    }
    let mut blocks: Vec<LoadedBlock> = Vec::new();
    for spec in &WIKIDATA_BLOCKS {
        let path = format!("{}/{}", ADMITTED_DIR, spec.file);
        let l = fs::read(&path).unwrap_or_else(|e| panic!("failed reading {path}: {e}"));
        assert_eq!(l.len(), spec.expected_n, "n mismatch on {}", spec.name);
        let (runs, _) = extract_runs(&l);
        assert_eq!(runs.len(), spec.expected_r, "r mismatch on {}", spec.name);
        blocks.push(LoadedBlock {
            name: spec.name.to_string(),
            l,
            orig_ptr: spec.primary_index,
            is_wikidata: true,
        });
    }
    // Add 2 non-Wikidata synthetic blocks (JSON API log + syslog)
    let syn_json_text = make_synthetic_json_log(480); // ~85 KiB
    let (syn_json_l, syn_json_p) = direct_bwt(&syn_json_text);
    blocks.push(LoadedBlock {
        name: "syn-json-log".to_string(),
        l: syn_json_l,
        orig_ptr: syn_json_p,
        is_wikidata: false,
    });
    let syn_sys_text = make_synthetic_syslog(900); // ~60 KiB
    let (syn_sys_l, syn_sys_p) = direct_bwt(&syn_sys_text);
    blocks.push(LoadedBlock {
        name: "syn-syslog".to_string(),
        l: syn_sys_l,
        orig_ptr: syn_sys_p,
        is_wikidata: false,
    });

    // ------------------------------------------------------------------------
    // PART 4: Step B1 & B2 Failure Diagnostics by Width Tier & 7 Text Classes
    // ------------------------------------------------------------------------
    println!("--- PART 3: Stepanov Steps B1 & B2 Failure Diagnostics (All 6 Wikidata + 2 Non-Wikidata) ---");
    println!(
        "{:<13} {:>7} {:>7} {:>4} {:>7} | {:>5} {:>5} {:>5} {:>5} | {:>5} {:>5} {:>5} | {:>7}",
        "block", "n", "r", "|S|", "r/|S|",
        "w1%", "2-16%", ">16%", "thin%",
        "G:w1%", "G:th%", "S:th%", "JK:all%"
    );
    for blk in &blocks {
        let (text, pos_of_row) = plain_invert_with_pos(&blk.l, blk.orig_ptr);
        let cls = classify_text(&text);
        let (runs, _) = extract_runs(&blk.l);
        let n = blk.l.len();
        let mut seen = [false; 256];
        for &b in &blk.l {
            seen[b as usize] = true;
        }
        let sigma = seen.iter().filter(|&&x| x).count();

        // Width tier & class breakdown
        let mut row_w1 = 0usize;
        let mut row_2_16 = 0usize;
        let mut row_17p = 0usize;
        let mut cls_all = [0usize; 7];
        let mut cls_w1 = [0usize; 7];
        let mut cls_2_16 = [0usize; 7];
        let mut cls_17p = [0usize; 7];
        let mut b2_breaks_by_cls = [0usize; 7];

        // Step B2: LF(x) != (LF(x-1) + 1) mod n -> exact run starts!
        for r in &runs {
            let tier = if r.width == 1 {
                row_w1 += 1;
                0
            } else if r.width <= 16 {
                row_2_16 += r.width as usize;
                1
            } else {
                row_17p += r.width as usize;
                2
            };
            // Breakpoint at run start
            let start_c = cls[pos_of_row[r.start as usize] as usize] as usize;
            b2_breaks_by_cls[start_c] += 1;
            for row in r.start..(r.start + r.width) {
                let c = cls[pos_of_row[row as usize] as usize] as usize;
                cls_all[c] += 1;
                match tier {
                    0 => cls_w1[c] += 1,
                    1 => cls_2_16[c] += 1,
                    _ => cls_17p[c] += 1,
                }
            }
        }
        let thin_rows = row_w1 + row_2_16;
        let guid_w1 = if row_w1 > 0 { 100.0 * cls_w1[0] as f64 / row_w1 as f64 } else { 0.0 };
        let guid_th = if thin_rows > 0 { 100.0 * (cls_w1[0] + cls_2_16[0]) as f64 / thin_rows as f64 } else { 0.0 };
        let sha_th = if thin_rows > 0 { 100.0 * (cls_w1[1] + cls_2_16[1]) as f64 / thin_rows as f64 } else { 0.0 };
        let jk_all = 100.0 * cls_all[2] as f64 / n as f64;
        println!(
            "{:<13} {:>7} {:>7} {:>4} {:>7.1} | {:>5.1} {:>5.1} {:>5.1} {:>5.1} | {:>5.1} {:>5.1} {:>5.1} | {:>7.1}",
            blk.name,
            n,
            runs.len(),
            sigma,
            runs.len() as f64 / sigma as f64,
            100.0 * row_w1 as f64 / n as f64,
            100.0 * row_2_16 as f64 / n as f64,
            100.0 * row_17p as f64 / n as f64,
            100.0 * thin_rows as f64 / n as f64,
            guid_w1,
            guid_th,
            sha_th,
            jk_all,
        );

        // Detailed 7-class B2 breakpoint & width-tier table for Wikidata blocks
        if blk.is_wikidata {
            print!("   [B2 breaks/class % of r]:");
            for c in 0..7 {
                print!(" {}={:.1}%", CLASS_NAMES[c], 100.0 * b2_breaks_by_cls[c] as f64 / runs.len() as f64);
            }
            println!();
            print!("   [2..16 rows/class %]:    ");
            for c in 0..7 {
                print!(" {}={:.1}%", CLASS_NAMES[c], 100.0 * cls_2_16[c] as f64 / row_2_16.max(1) as f64);
            }
            println!();
        }
    }

    // ------------------------------------------------------------------------
    // PART 5: Step B3 Failure (MSC 37 Full Zorich-Accelerated RV vs Open<16>)
    // ------------------------------------------------------------------------
    println!("\n--- PART 4: Step B3 Diagnostics — MSC 37 Full Zorich-Accelerated RV vs Open<16> ---");
    println!(
        "{:<13} | {:>8} {:>7} {:>7} {:>5} {:>5} {:>5} {:>6} {:>7} | {:>7} {:>6} {:>6} {:>6}",
        "block",
        "FullStp", "q=1%", "q>=2%", "w1%", "2-16%", ">16%", "Rules", "MaxAct",
        "O16Stp", "O16q1%", "O16Rul", "O16Bnd"
    );
    for blk in &blocks {
        let (runs, _) = extract_runs(&blk.l);
        // Full Zorich-accelerated RV (all runs except mark eliminated!)
        let elim_all = vec![true; runs.len()];
        let (pieces_all, _) = build_pieces_from_runs(&runs, blk.orig_ptr, &elim_all);
        let mut dag_all = WordDag::new();
        let (_bands_all, st_all) = jsv_trace_morphism(blk.l.len() as u32, &pieces_all, &mut dag_all, true);

        // Open<16> thick-only Zorich-accelerated RV
        let elim16: Vec<bool> = runs.iter().map(|r| r.width > 16).collect();
        let (pieces16, _) = build_pieces_from_runs(&runs, blk.orig_ptr, &elim16);
        let mut dag16 = WordDag::new();
        let (_bands16, st16) = jsv_trace_morphism(blk.l.len() as u32, &pieces16, &mut dag16, true);

        let full_q1 = 100.0 * st_all.peels_q1 as f64 / st_all.steps.max(1) as f64;
        let full_q2p = 100.0 * (st_all.peels_q2 + st_all.peels_q3 + st_all.peels_q4p) as f64 / st_all.steps.max(1) as f64;
        let full_w1 = 100.0 * st_all.steps_w1 as f64 / st_all.steps.max(1) as f64;
        let full_w2_16 = 100.0 * st_all.steps_w2_16 as f64 / st_all.steps.max(1) as f64;
        let full_w17p = 100.0 * st_all.steps_w17p as f64 / st_all.steps.max(1) as f64;
        let o16_q1 = 100.0 * st16.peels_q1 as f64 / st16.steps.max(1) as f64;
        println!(
            "{:<13} | {:>8} {:>6.2}% {:>6.2}% {:>4.1}% {:>4.1}% {:>4.1}% {:>6} {:>7} | {:>7} {:>5.1}% {:>6} {:>6}",
            blk.name,
            st_all.steps, full_q1, full_q2p, full_w1, full_w2_16, full_w17p, st_all.rules_created, st_all.max_active_nodes,
            st16.steps, o16_q1, st16.rules_created, st16.bands_out
        );
    }

    // ------------------------------------------------------------------------
    // PART 6: MSC 18 JSV Trace Vanishing II Verification + Stratified Leverage Tower
    // ------------------------------------------------------------------------
    println!("\n--- PART 5: MSC 18 JSV Categorical Trace — Vanishing II Exactness & Stratified Leverage Tower ---");
    for blk in &blocks {
        let (runs, _) = extract_runs(&blk.l);
        let n = blk.l.len();
        print!("  {:<13} Vanishing-II:", blk.name);
        for &l1 in &[8u32, 4, 2, 1] {
            let (ok, m1, st1, st2_sum, b1) = verify_vanishing_ii(&blk.l, &runs, blk.orig_ptr, l1, 16);
            assert!(ok, "Vanishing II failed on {} L1={}", blk.name, l1);
            print!(" L1={}:EXACT(m={},st1={},st2={},B={})", l1, m1, st1, st2_sum, b1);
        }
        println!();

        // Stratified Trace Leverage Tower: L = 16, 8, 4, 2, 1
        let mut prev_m = n;
        let mut prev_steps = 0usize;
        print!("     Leverage Tower:");
        for &lim in &[16u32, 8, 4, 2, 1] {
            let elim: Vec<bool> = runs.iter().map(|r| r.width > lim).collect();
            let (pieces, ret) = build_pieces_from_runs(&runs, blk.orig_ptr, &elim);
            let mut dag = WordDag::new();
            let (bands, st) = jsv_trace_morphism(n as u32, &pieces, &mut dag, false);
            let elim_delta = prev_m.saturating_sub(ret.len());
            let step_delta = st.steps.saturating_sub(prev_steps).max(1);
            let leverage = elim_delta as f64 / step_delta as f64;
            print!(" [L={}:m={},B={},dRow/dStp={:.2}]", lim, ret.len(), bands.len(), leverage);
            prev_m = ret.len();
            prev_steps = st.steps;
        }
        println!();
    }

    // ------------------------------------------------------------------------
    // PART 7: MSC 18 / MSC 82 Copy-Valley Localization (U_{iso, 2..16} & U_{mono, 2..16})
    // ------------------------------------------------------------------------
    println!("\n--- PART 6: MSC 18 Iso-Localization / MSC 82 MK1D Copy-Valley Decimation (2 <= w <= 16) ---");
    println!(
        "{:<13} | {:>7} {:>6} {:>6} | {:>7} {:>6} {:>6} {:>6} | {:>7} {:>6} | {:>6} {:>6}",
        "block",
        "m_O16", "B_O16", "D_O16",
        "m_Iso", "dm%", "B_Iso", "D_Iso",
        "m_Mono", "dmM%",
        "IsoSh%", "IsoPr%"
    );
    for blk in &blocks {
        let (text, pos_of_row) = plain_invert_with_pos(&blk.l, blk.orig_ptr);
        let cls = classify_text(&text);
        let (runs, _) = extract_runs(&blk.l);
        let n = blk.l.len();

        // Baseline Open<16>
        let elim16: Vec<bool> = runs.iter().map(|r| r.width > 16).collect();
        let (p16, _) = build_pieces_from_runs(&runs, blk.orig_ptr, &elim16);
        let mut dag16 = WordDag::new();
        let (b16, _) = jsv_trace_morphism(n as u32, &p16, &mut dag16, false);
        let rep16 = wire_and_emit_from_bands(&blk.l, &runs, blk.orig_ptr, &elim16, &b16, &dag16);
        assert_eq!(rep16.out, text, "Open<16> exactness failed on {}", blk.name);

        // Detect iso and mono copy valleys in 2..=16
        let (iso, mono) = detect_copy_valleys(&runs, 16);

        // Compute how many SHA1 thin rows (2..=16) are caught by `iso` (recall & precision!)
        let mut total_sha_thin = 0usize;
        let mut iso_rows = 0usize;
        let mut iso_sha_rows = 0usize;
        for (u, r) in runs.iter().enumerate() {
            if r.width <= 16 {
                for row in r.start..(r.start + r.width) {
                    let is_sha = cls[pos_of_row[row as usize] as usize] == TextClass::Sha1;
                    if is_sha {
                        total_sha_thin += 1;
                    }
                    if iso[u] {
                        iso_rows += 1;
                        if is_sha {
                            iso_sha_rows += 1;
                        }
                    }
                }
            }
        }
        let iso_sha_recall = if total_sha_thin > 0 { 100.0 * iso_sha_rows as f64 / total_sha_thin as f64 } else { 0.0 };
        let iso_sha_prec = if iso_rows > 0 { 100.0 * iso_sha_rows as f64 / iso_rows as f64 } else { 0.0 };

        // Trace Open<16> + U_{iso, 2..16}
        let mut elim_iso = elim16.clone();
        for i in 0..runs.len() {
            if iso[i] {
                elim_iso[i] = true;
            }
        }
        let (p_iso, _) = build_pieces_from_runs(&runs, blk.orig_ptr, &elim_iso);
        let mut dag_iso = WordDag::new();
        let (b_iso, _) = jsv_trace_morphism(n as u32, &p_iso, &mut dag_iso, false);
        let rep_iso = wire_and_emit_from_bands(&blk.l, &runs, blk.orig_ptr, &elim_iso, &b_iso, &dag_iso);
        assert_eq!(rep_iso.out, text, "JSV+Iso exactness failed on {}", blk.name);

        // Trace Open<16> + U_{mono, 2..16}
        let mut elim_mono = elim16.clone();
        for i in 0..runs.len() {
            if mono[i] {
                elim_mono[i] = true;
            }
        }
        let (p_mono, _) = build_pieces_from_runs(&runs, blk.orig_ptr, &elim_mono);
        let mut dag_mono = WordDag::new();
        let (b_mono, _) = jsv_trace_morphism(n as u32, &p_mono, &mut dag_mono, false);
        let rep_mono = wire_and_emit_from_bands(&blk.l, &runs, blk.orig_ptr, &elim_mono, &b_mono, &dag_mono);
        assert_eq!(rep_mono.out, text, "JSV+Mono exactness failed on {}", blk.name);

        let dm_iso = 100.0 * (rep16.retained_m as f64 - rep_iso.retained_m as f64) / rep16.retained_m as f64;
        let dm_mono = 100.0 * (rep16.retained_m as f64 - rep_mono.retained_m as f64) / rep16.retained_m as f64;
        println!(
            "{:<13} | {:>7} {:>6} {:>6} | {:>7} {:>5.1}% {:>6} {:>6} | {:>7} {:>5.1}% | {:>5.1}% {:>5.1}%",
            blk.name,
            rep16.retained_m, b16.len(), rep16.distinct_words,
            rep_iso.retained_m, dm_iso, b_iso.len(), rep_iso.distinct_words,
            rep_mono.retained_m, dm_mono,
            iso_sha_recall, iso_sha_prec
        );
        println!(
            "     [Dict bytes & inline<=15%]: O16={} B ({:.1}%), Iso={} B ({:.1}%), Mono={} B ({:.1}%)",
            rep16.spelling_bytes, rep16.pct_inline15,
            rep_iso.spelling_bytes, rep_iso.pct_inline15,
            rep_mono.spelling_bytes, rep_mono.pct_inline15
        );
    }

    // ------------------------------------------------------------------------
    // PART 8: Assembled Exact Transplant Chain Benchmark (1T / 2T / 4W)
    // ------------------------------------------------------------------------
    println!("\n--- PART 7: Assembled Exact Chain Benchmark (1T, 2T, 4W — All 6 Wikidata + Sturmian-832K + Synthetics) ---");
    // Add 832,040-byte Fibonacci Sturmian block for benchmark!
    let fib_l = make_sturmian_column(514_229, 317_811, b'a', b'b');
    let fib_p = 307_854u32;
    let fib_ref = plain_invert(&fib_l, fib_p);
    let mut bench_blocks: Vec<(&str, &[u8], u32, Vec<u8>)> = Vec::new();
    for blk in &blocks {
        let ref_t = plain_invert(&blk.l, blk.orig_ptr);
        bench_blocks.push((blk.name.as_str(), blk.l.as_slice(), blk.orig_ptr, ref_t));
    }
    bench_blocks.push(("fib-sturm-832k", fib_l.as_slice(), fib_p, fib_ref));

    // Verify 100% byte-exactness first on all bench blocks
    let mut o_tmp = Vec::new();
    for (name, l, p, ref_t) in &bench_blocks {
        for (kname, k) in [("Open16-Ser", &kernel_ser), ("Chain-BI", &kernel_bi), ("Chain-BII", &kernel_bii)] {
            k.invert_timed(l, *p, &mut o_tmp);
            assert_eq!(&o_tmp, ref_t, "{kname} failed exactness on {name}");
        }
    }
    println!("  All 3 assembled chains verified 100% BYTE-EXACT on all 6 Wikidata + 2 synthetic + Sturmian-832K blocks.");

    // 1T Median-of-5 per-block benchmark
    println!(
        "\n  1T Per-Block Median (5 runs, us) — Construct / Emit / Total:"
    );
    println!(
        "  {:<15} | {:>19} | {:>19} | {:>19} | {:>7} {:>7}",
        "block", "Open16-Ser (c/e/t)", "Chain-BI (c/e/t)", "Chain-BII (c/e/t)", "BI/Ser", "BII/BI"
    );
    let mut sum_ser = 0.0f64;
    let mut sum_bi = 0.0f64;
    let mut sum_bii = 0.0f64;
    let mut sum_ser_emit = 0.0f64;
    let mut sum_bi_emit = 0.0f64;
    let mut sum_bii_emit = 0.0f64;

    for (name, l, p, _) in &bench_blocks {
        let run_med = |k: &FastKernel| -> (f64, f64, f64) {
            let mut cs = Vec::new();
            let mut es = Vec::new();
            let mut ts = Vec::new();
            let mut buf = Vec::with_capacity(l.len() + 16);
            for _ in 0..5 {
                let m = k.invert_timed(l, *p, &mut buf);
                cs.push(m.construct_us);
                es.push(m.emit_us);
                ts.push(m.total_us);
            }
            cs.sort_by(|a, b| a.total_cmp(b));
            es.sort_by(|a, b| a.total_cmp(b));
            ts.sort_by(|a, b| a.total_cmp(b));
            (cs[2], es[2], ts[2])
        };
        let (sc, se, st) = run_med(&kernel_ser);
        let (bc, be, bt) = run_med(&kernel_bi);
        let (mc, me, mt) = run_med(&kernel_bii);
        if name.starts_with("block-") {
            sum_ser += st;
            sum_bi += bt;
            sum_bii += mt;
            sum_ser_emit += se;
            sum_bi_emit += be;
            sum_bii_emit += me;
        }
        println!(
            "  {:<15} | {:>6.0}/{:>5.0}/{:>6.0} | {:>6.0}/{:>5.0}/{:>6.0} | {:>6.0}/{:>5.0}/{:>6.0} | {:>6.3}x {:>6.3}x",
            name, sc, se, st, bc, be, bt, mc, me, mt, bt / st, mt / bt
        );
    }
    println!(
        "  6-Block Wikidata 1T Sum: Ser={:.0} us (emit {:.0}), Chain-BI={:.0} us (emit {:.0}, {:.3}x total / {:.3}x emit), Chain-BII={:.0} us (emit {:.0}, {:.3}x vs BI / {:.3}x emit vs BI)",
        sum_ser, sum_ser_emit,
        sum_bi, sum_bi_emit, sum_bi / sum_ser, sum_bi_emit / sum_ser_emit,
        sum_bii, sum_bii_emit, sum_bii / sum_bi, sum_bii_emit / sum_bi_emit
    );

    // 2T and 4W concurrent throughput across the 6 Wikidata blocks (median of 5 waves)
    for workers in [2usize, 4usize] {
        let run_wave = |k: &FastKernel| -> f64 {
            let mut times = Vec::new();
            for _ in 0..5 {
                let t0 = Instant::now();
                std::thread::scope(|s| {
                    for w in 0..workers {
                        let brefs = &blocks[..6];
                        s.spawn(move || {
                            let mut buf = Vec::with_capacity(900_016);
                            for i in 0..6 {
                                let blk = &brefs[(i + w) % 6];
                                k.invert_timed(&blk.l, blk.orig_ptr, &mut buf);
                            }
                        });
                    }
                });
                times.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            times.sort_by(|a, b| a.total_cmp(b));
            times[2]
        };
        let t_ser = run_wave(&kernel_ser);
        let t_bi = run_wave(&kernel_bi);
        let t_bii = run_wave(&kernel_bii);
        println!(
            "  {}W Wave (6 Wikidata blocks/worker, median of 5): Open16-Ser={:.2} ms, Chain-BI={:.2} ms ({:.3}x), Chain-BII={:.2} ms ({:.3}x vs Ser, {:.3}x vs BI)",
            workers, t_ser, t_bi, t_bi / t_ser, t_bii, t_bii / t_ser, t_bii / t_bi
        );
    }
}
