from pathlib import Path

base_mod = Path("/tmp/cks_bench/master_a1/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/master_a1/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/master_a1/outq.rs").read_text()

# --- CANDIDATE 18A: `cand_hotwire` (Reverse/Hot-Byte-Last Wire Gauge) ---
# Tests whether wiring cold bytes first and hot JSON bytes (`"`, `,`, `:`, `0..9`, `a..z`) LAST
# warms the 256 KiB L2 / 2 MiB L2 cache for the start of `emit` at zero extra instructions!
hw_mod = base_mod.replace(
    "            let mut k = 0;\n            for a in 0..256 {",
    """            let mut byte_starts = [0u32; 256];
            {
                let mut acc = 0u32;
                for a in 0..256 {
                    byte_starts[a] = acc;
                    acc += thin[a];
                }
            }
            // Sort bytes ascending by `thin[a]` so the largest/hottest buckets are wired LAST (MRU in L2)!
            let mut order: [u8; 256] = std::array::from_fn(|i| i as u8);
            order.sort_unstable_by_key(|&b| thin[b as usize]);
            for &ab in &order {
                let a = usize::from(ab);
                if thin[a] == 0 { continue; }
                let mut k = byte_starts[a];"""
)
d_hw = Path("/tmp/cks_bench/cand_hotwire")
d_hw.mkdir(parents=True, exist_ok=True)
(d_hw / "mod.rs").write_text(hw_mod)
(d_hw / "endpoint.rs").write_text(base_ep)
(d_hw / "outq.rs").write_text(base_oq)

# --- CANDIDATE 18B: `cand_bipartite` (2-Adic Bipartite Independent-Set Contraction `E_{0->1}` with 4 KiB L1D Bigram Table) ---
# Mathematical construction:
# After `wire` builds `edges[0..m)`:
# When `!wide && bits <= 19` (always true on all admitted & Cap::dense blocks, since m <= 256,000 => bits <= 18!):
# High bit 31 (`0x8000_0000`) tags a contracted 2-byte literal bigram!
# Bits `bits .. 31` (`13` bits when `bits == 18`, i.e. `8,192` slots; we use a `2,048`-entry `[u8; 2]` table = `4 KiB` in L1D,
# direct-mapped by `((c1 * 167) ^ c2) & 2047`!)
# Sweep `u` in even indices `0, 2, 4, .. < m`:
# Let `eu = edges[u]`, `v = eu & mask`, `cu = eu >> bits`.
# If `(v & 1) == 1 && v != mark && cu < 256`:
#   Because `u` is EVEN and `v` is ODD, NO two such edges `(u -> v)` share an endpoint (`100% disjoint matching E_{0->1}`)!
#   So reading `ev = edges[v as usize]` reads an UNMODIFIED entry (AND all `u = 0, 2, 4, ...` reads are 100% data-independent,
#   issuing at full OoO MLP mu ~ 8-12)!
#   If `cv = ev >> bits` is ALSO `< 256` (`literal`):
#     Lookup/insert `(cu as u8, cv as u8)` in `bigrams: [[u8; 2]; 2048]` (`4 KiB` L1D)!
#     If slot matches or was empty:
#       `edges[u] = 0x8000_0000 | ((b_slot as u32) << bits) | (ev & mask)`!
# Then in `emit`:
#   If `(edge & 0x8000_0000) != 0`:
#     `let bg = bigrams[((edge & 0x7FFF_FFFF) >> bits) as usize];`
#     `out[at] = bg[0]; out[at + 1] = bg[1]; at += 2;` (2-byte store, skipping vertex `v` completely!)

bp_mod = base_mod.replace(
    "    lens: Vec<u8>,\n}",
    "    lens: Vec<u8>,\n    bigrams: Vec<[u8; 2]>,\n}"
).replace(
    "        let lens = self.lens.capacity();",
    "        let lens = self.lens.capacity() + 2 * self.bigrams.capacity();"
)

bp_contract_code = """        // 2-Adic Bipartite Independent-Set Contraction `E_{0 -> 1}`:
        if !wide && bits <= 19 && m >= 65_536 {
            if self.bigrams.len() < 2048 {
                self.bigrams.resize(2048, [0, 0]);
            }
            // Bitvec of occupied bigram slots (32 u64s = 256 bytes on stack!)
            let mut bg_used = [0u64; 32];
            let bg_tab = &mut self.bigrams[..2048];
            let mask = (1u32 << bits) - 1;
            let edges_m = &mut self.edges[..m as usize];
            // Process in chunks of 8 even indices so LLVM unrolls 8 independent `edges_m[v]` loads in parallel!
            let mut u = 0usize;
            while u < m as usize {
                let eu = edges_m[u];
                let v = eu & mask;
                let cu = eu >> bits;
                if (v & 1) == 1 && v != mark && cu < 256 {
                    let ev = edges_m[v as usize];
                    let cv = ev >> bits;
                    if cv < 256 {
                        let b0 = cu as u8;
                        let b1 = cv as u8;
                        let bslot = ((usize::from(b0).wrapping_mul(173) ^ usize::from(b1)) & 2047);
                        let word_i = bslot >> 6;
                        let bit_i = 1u64 << (bslot & 63);
                        let ok = if (bg_used[word_i] & bit_i) == 0 {
                            bg_used[word_i] |= bit_i;
                            bg_tab[bslot] = [b0, b1];
                            true
                        } else {
                            bg_tab[bslot] == [b0, b1]
                        };
                        if ok {
                            edges_m[u] = 0x8000_0000 | ((bslot as u32) << bits) | (ev & mask);
                        }
                    }
                }
                u += 2;
            }
        }
        (stats, mark, bits)"""

assert "        (stats, mark, bits)" in bp_mod
bp_mod = bp_mod.replace("        (stats, mark, bits)", bp_contract_code, 1)

old_bp_walk = """        emit(
            &self.edges[..m],
            &self.extra,
            &self.spelling,
            &self.offsets,
            &self.inline16,
            &self.lens,
            n,
            mark,
            bits,
            out,
        );"""

new_bp_walk = """        emit(
            &self.edges[..m],
            &self.extra,
            &self.spelling,
            &self.offsets,
            &self.inline16,
            &self.lens,
            &self.bigrams,
            n,
            mark,
            bits,
            out,
        );"""
bp_mod = bp_mod.replace(old_bp_walk, new_bp_walk)

old_bp_emit = """fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    let mask = (1 << bits) - 1;
    let mut j = mark;
    // (the port's one change to the walk) the cycle through the marked row visits each retained
    // row once at most, so a valid construction returns to it within `edges.len()` steps; the
    // bound and the clamp to `n` make every walk end, whatever the construction: a wrong one
    // gives a wrong block, which the block's CRC rejects, never a walk that grows without end
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        } as usize;
        j = edge & mask;
        let len = lens[code];
        if len != LONG {
            // `at < n` here, so the copy ends within `n + 16`
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let w = code - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }"""

new_bp_emit = """fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    bigrams: &[[u8; 2]],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    let mask = (1 << bits) - 1;
    let mut j = mark;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        j = edge & mask;
        if (edge & 0x8000_0000) != 0 {
            let bslot = ((edge & 0x7FFF_FFFF) >> bits) as usize;
            let bg = bigrams[bslot];
            out[at] = bg[0];
            out[at + 1] = bg[1];
            at += 2;
        } else {
            let code = if extra.is_empty() {
                edge >> bits
            } else {
                extra[j as usize] // note: `wide` disables bigrams so extra is empty when bigrams used
            } as usize;
            let len = lens[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                let w = code - 256;
                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
        }
        if j == mark || at >= n {
            break;
        }
    }"""
# Wait: fix `extra[j as usize]` in `else` branch for `wide` case (save `old_j = j` before `j = edge & mask`!):
new_bp_emit = new_bp_emit.replace(
    "        let edge = edges[j as usize];\n        j = edge & mask;",
    "        let old_j = j as usize;\n        let edge = edges[old_j];\n        j = edge & mask;"
).replace("extra[j as usize]", "extra[old_j]")

assert old_bp_emit in bp_mod
bp_mod = bp_mod.replace(old_bp_emit, new_bp_emit)

d_bp = Path("/tmp/cks_bench/cand_bipartite")
d_bp.mkdir(parents=True, exist_ok=True)
(d_bp / "mod.rs").write_text(bp_mod)
(d_bp / "endpoint.rs").write_text(base_ep)
(d_bp / "outq.rs").write_text(base_oq)

print("Generated cand_hotwire and cand_bipartite successfully.")
