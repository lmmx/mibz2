# Held-out Wikidata windows for P74-C: registration (frozen 2026-10-31, before any timing)

**Label:** same-dump held-out Wikidata, *not* independent-source validation.
- **The dump:** the windows come from the same dated dump as the fitted corpus,
  `wikidata-20261019-all.json.bz2` (sha1 `a05b8b80…`).
- **External generalisation** needs a separately sourced corpus: a different dump date or
  a different dataset.

**Selection rule:** 12 windows of 1,048,576 bytes at ⌊(4i+3)/48 · 103,488,697,311⌋,
i = 0..11, midway between the fitted windows at (2i+1)/24. The positions were fixed before
fetching. Per-window SHA-256 values are in `spec.json`. The corpus is built by the frozen
harness's `--windows` (bb17a24), which keeps each window's whole blocks unchanged.

**Corpus:** `wikidata-independent.bz2`:
- SHA-256 `7f5c9a0044e5a30589bf343b9b31e698849b2acc69461a55c97b550193128b45`;
- 11,965,711 bytes;
- 473 blocks, 235,800,351 MTF L bytes.

**Done before registration (correctness and selection only, untimed),** at the frozen code
and policy (bb17a24, inclusive-trigger rule):
- `--check`: all arms identical;
- selector audit: 211 bypassed, 262 P74 to the block's end, 0 handoffs.

No parameter, code or rule was changed after seeing these.

**What it can test:** P74's gain out of sample, and that no Wikidata block is falsely
switched. It cannot test the online guard's economics, since it has 0 handoffs. That needs
the frozen adversarial `.bz2` suite, with its handoffs verified before timing.

**Acceptance rows** (to be fixed in the experiment's pre-registration, before timing): the
same as the fitted corpus's Wikidata checkpoint row, reported separately and never pooled.
