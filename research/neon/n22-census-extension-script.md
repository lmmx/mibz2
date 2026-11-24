# N-22: Self-Contained 232-Block Census Extension Script (`N-0`, `M-4`/`N-18`, `M-8`)

*Cycle 5 handoff answering Coordinator Note #12 (`origin/claude/sharp-meitner-cx0v78` `c4847b8`): prepare `research/neon/scripts/census_extension.py` to run on Louis's workstation across all 232 distinct census blocks, verified exact against all published 6-block numbers and **not** run on the corpus.*

**Script:** [`research/neon/scripts/census_extension.py`](scripts/census_extension.py)

## 1. Handoff Specification & CLI Usage

- **Dependencies:** Python 3.10+ standard library **only** (`argparse`, `collections`, `concurrent.futures`, `hashlib`, `json`, `math`, `pathlib`, `re`, `subprocess`, `sys`, `tempfile` — zero third-party packages, zero imports from sibling repository scripts).
- **Input format (`data/admitted-bwt` layout):**
  - Directory `<bwt_dir>` containing `manifest.json` (`{"blocks": [{"file": "...", "primary_index": ...}, ...]}`; also accepts `"orig_ptr"` or `"p"` as fallback keys, and validates `"n"`, `"r"`, and `"sha256"` whenever present) and one raw `.L` file per block (1 byte per BWT row, before RLE1).
- **CLI invocation (for workstation recipe):**
  ```bash
  python3 research/neon/scripts/census_extension.py /path/to/bwt_dir -o census_extension.tsv --jobs 10
  ```
  *(Deterministic row order matching `manifest["blocks"]` is preserved regardless of `--jobs`.)*
- **Self-test on the 6 admitted blocks (run here; corpus NOT touched):**
  ```bash
  python3 research/neon/scripts/census_extension.py --verify-admitted --jobs 6
  ```

## 2. Output TSV Schema (52 Columns per Block, Joinable to `#805` `census.tsv` on `(n, p, r)`)

| Column group | Columns | Source / Meaning |
|---|---|---|
| **Block geometry** | `block`, `n`, `p`, `r`, `m16` | Filename, row count, `primary_index`, maximal runs `r`, Open retained rows `m₁₆` (`LIMIT = 16`) |
| **`N-0` / `N-2` (`σ_hex` & `Pglue`)** | `h1`, `h1R`, `h1N`, `h1R_share`, `h1N_share`, `pglue_w1_switches`, `all_switches` | Width-1 runs `h₁`, hex width-1 `h₁R` (`[0-9a-fA-F]`), non-hex `h₁N`, shares, and orbit `R↔N` boundary switch counts |
| **Amended `N-0` (`N-8` & `M-7`)** | `novel_hex`, `repeated_hex`, `non_hex`, `novel_hex_share`, `repeated_hex_share`, `non_hex_share`, `novel_hex6`, `repeated_hex6`, `short_hex_1_5`, `novel_hex6_share`, `repeated_hex6_share`, `short_hex_1_5_share`, `rep_sha1_occ`, `rep_sha1_w1`, `nov_sha1_occ` | Exact 3-part novelty split under `Def1` (`[0-9a-fA-F]+`) and `Def2` (`[0-9a-fA-F]{6,}`), plus `M-7` 40-hex SHA1 width-1 verification |
| **`M-4` (`N-9`) & `N-18` (`ν̂` quantiles)** | `valid_flips_256`, `f_disj`, `u_all_flips`, `u_all_flips_j1`, `hex_w1_ep_cov`, `nonhex_w1_ep_cov`, `nu_q01`, `nu_q05`, `nu_q10`, `nu_q25`, `nu_q50`, `nu_q75`, `nu_q90`, `nu_hex_p10`, `nu_hex_p50`, `nu_hex_p90`, `nu_nonhex_p10`, `nu_nonhex_p50`, `nu_nonhex_p90` | Valid Type-A/B flips (`SPAN_CAP = 256`), optimal disjoint-span flips `\|ℱ_disj\|`, `\|U_all-flips\|`, `J₁` critical observation budgets `q_{1%}..q_{90%}` (`q_{50%}` = Theorem N′ `δ = 1/2` wall), and Stream R / Stream N deciles |
| **`M-8` (`N-15` i9-10900K `𝓗`)** | `d_l2`, `d_l2_over_m16`, `d_l2_over_h1`, `d_l1d`, `d_l3_cold`, `d_l3_cores_10w` | Exact `emit_bidirectional` 64B-line LRU cache-miss counts at L1d (`32 KiB`), L2 (`256 KiB`), and L3/worker at `10W` (`2,048 KiB`, cold and co-resident with `out[]`) |

## 3. Six-Block Verification Receipt (`[measured]`)

Tested both via staged temporary directory (`<bwt_dir>` + `-o out.tsv` + `-j 6`) and via `--verify-admitted`:
- **Result:** `PASS` — **6/6 admitted blocks matched 100% of all `228` published exact figures** across `N-0`/`N-2` (`h₁, h₁R, h₁N, Pglue`), `N-8` (`novel_hex, repeated_hex, non_hex`, `Def2`, `SHA1`), `N-9` (`valid_flips_256, f_disj, u_all_flips`, stream deciles), `N-18` (`nu_q01..nu_q90`, including `q_{50%} = 6,903 / 8,761 / 9,988 / 14,120 / 12,974 / 9,684`), and `N-15` (`d_l2 = 14,630 / 27,293 / 42,608 / 86,903 / 95,695 / 41,813`, `d_l1d`, `d_l3_cold`, `d_l3_cores_10w`).
- **Corpus constraint:** Not run on `corpora/wikidata.bz2` (absent locally and excluded by assignment).
