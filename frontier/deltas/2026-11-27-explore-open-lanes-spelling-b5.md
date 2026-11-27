claim: explore
by: antigravity
outcome: pass
evidence: docs/journal/2026-11-27-explore/open-lanes-spelling-and-b5-dissection.md; examples/open_lanes_b5_dissection.rs; frontier/claims/lanes-disjoint-slabs-build.toml

Cold `objdump -d` and 7-repeat warm + L3-cold dissection of `src/open/lanes.rs` (`#1110` `--features forward-anchor-lanes`, Track A) and `src/open.rs` setup/classify/clear outside `A6a/b/c/7/9` (Track B) across baseline x86-64 and JCC-32B-aligned native builds (233/233 Wikidata Open blocks + 1/1 Open `runs` block 100% byte+CRC exact across all 8 arms `L0..L6b`; 0 Open blocks on `text`/`random`/`records`/`pattern`/`onebyte`).

- **Track A (`src/open/lanes.rs`) — DECISIVE PASS:**
  - `objdump -d` of `<Kernel>::walk` exposed 3,728 insns (`0x448` frame, `1,178` stack ops, `69` `vzeroupper`s on JCC-32B) from `#[inline(always)] run<K>`, 20 bounds/overflow checks per 4-lane step, and `1,718,999` `#[cold] word_slow` calls (`99.70%` of `LONG` words fit in the current 32 KiB slab; only `16,000` actual slab crossings = `107.4x` call inflation).
  - **100% safe Rust `L3_disjoint_safe`** (`#[inline(never)] run<K>` + per-lane owned `Box<[u8; 32768]>` slabs + inline `LONG`-fits-in-slab + lazy slab pool, **0 `unsafe`**) cuts `run<4>` to `531..567` insns (`0x98` frame, `0` `vzeroupper`), cuts `lanes::walk` **-17.13% / -22.96%** (base warm/cold) and **-12.23% / -18.72%** (JCC-32B warm/cold), cuts `lanes::walk + A9` **-15.53% / -19.82%** (base) and **-10.67% / -16.91%** (JCC-32B, all 4/4 disjoint FASTER, ~`-11%` of `W_open` L3-cold), captures 99–100% of the `unsafe` zero-bc ceiling (`L4`), and shrinks per-worker Arena by **-95.8 KiB** (`1,056.8 -> 961.0 KiB`). Admitted as `lanes-disjoint-slabs-build` (`status = "idea"`, P2, `src/open/lanes.rs`, zero `unsafe`).
  - **Fused forward AVX2 RLE1 + 64 KiB CRC32 on ordered slab slices (`L6a_safe_walk+avx2_RLE`)** pushes `lanes::walk + A9` to **-18.40% / -25.50%** (base) and **-15.98% / -23.28%** (JCC-32B, 4/4 disjoint FASTER, **`-14.9%..-15.4%` of `W_open` L3-cold**) at `-95.8 KiB`/worker LESS memory.
  - Negative ablations recorded in `frontier/dead-ends.tsv`: scalar safe fused RLE1 (`L5`, `+33..+87%` SLOWER) and `unsafe` `get_unchecked` walk over disjoint-slab safe `L3` (`L4`, `<= 0.23%` L3-cold delta).
- **Track B (`src/open.rs` setup/classify/clear outside `A6a/b/c/7/9`) — HONEST STOP:**
  - Total non-`A6a/b/c/7/9` overhead is only **`0.64%–1.24%` of `W_open`** (`4.04–8.23 ms`, below the `>= 3.0%` gate): recycled `free_out` `out.resize` is `0.00 ms` (preserving `out.len()` avoids `19.7–25.4 ms` / `3.1–3.3%` L3-cold `memset(0)`), `finish()` strip-setup + `image_order` is `4.03–4.22 ms` (`0.53–0.73%`), and `within_cap` on 5 dense blocks is `0.00–4.01 ms`. Recorded in `frontier/dead-ends.tsv`.
