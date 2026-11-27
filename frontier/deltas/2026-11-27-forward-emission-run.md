claim: forward-emission-run
by: antigravity
outcome: partial
evidence: docs/journal/2026-11-27-forward-emission/registration.md; #1110 (7a7a207a); #1050

## Summary
Implemented the executable shallow-safe i9-10900K kill-test recipe (`just bench-forward-lanes`) for `forward-emission-run`, implementing `docs/journal/2026-11-27-forward-emission/registration.md` (§2, §3, §5) verbatim with zero threshold changes (RULES C2).

## Recipe & Hardening (`.just/bench.just`, `bench/forward_lanes.py`, `bench/test_forward_lanes.py`)
- **Shallow-checkout-safe pinned builds:** `checkout_commit()` fetches exact pinned `base` (`7beb5e9ba473`, default features) and `cand` (`7a7a207adfc2`, `--features forward-anchor-lanes`) via `git init` + `git fetch --no-tags --depth=1` directly from `origin`, reusing `examples/a6b_endpoint_run.rs` (forced Open route `lazy=true` at `k = 1` and `k = 8`, §5) and `examples/mtf_fold.rs` (`scan-oracle` + trace).
- **Isolated build targets & binary/quota guards:** `build_binaries()` compiles `base` and `cand` into separate `cargo-target-base` and `cargo-target-cand` directories under `RUSTUP_TOOLCHAIN=1.99.1` (`--release --locked`), enforces `assert_distinct_binaries()` on `a6b_endpoint_run` and `mtf_fold` SHA-256 digests (`VOID` on collision), and records/validates both `ab env` (`cpu_quota: null`) and `ab env --facts` (`{"state": "unlimited", "cpus": null}`) into `env.json` / `env-facts.json`.
- **Frozen §2–§3 design & verdict rule:** 15 interleaved AB/BA paired repeats across `wikidata x4` (`k = 1` forced Open, `k = 8` shipped, `k = 8` `S ideal` + matched `none` trace with `<= 5%` scan-residual gate) and `text` (`k = 1`), verifying libbzip2 SHA-256 on every repeat (`cand` mismatch/CRC -> `KILL`; `base` or harness violation -> `VOID`), scoring deterministic 10,000-sample paired bootstrap 95% CIs against §3 (`PASS`/`KILL`/`RESCUE`/`VOID`).
- **Recipe safety:** `.just/bench.just` preserves `bench-jcc-aligned-release`, writes under `target/forward-lanes/run-<UTC>`, writes `INCOMPLETE (exit $rc)` fallback on abnormal exit, runs `bench/bundle.sh`, and performs zero `$HOME` writes.

## Verification
- `python3 -W error -m unittest discover -s bench -p 'test_forward_lanes.py' -v`: 13/13 PASS (including real detached `git clone --depth=1 --single-branch --no-local` shallow checkout, isolated per-arm target dir check, identical-binary rejection, dual `cpu_quota` schema checks, and all §3 `PASS`/`KILL`/`RESCUE`/`VOID` branches).
- `ruff check` & `ruff format --check`: clean. Ready for independent C6 review (`status = "review"`); no queue entry or ready signal touched yet.
