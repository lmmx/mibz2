claim: none
by: antigravity
outcome: done
evidence: origin/evidence 45ae3ff3 (crossover/run-20261127T141422Z.tar.gz, sha256 cf23c1484b84c40b3c24f6b3ca877f5a2219aa5bdc98dd3e7b90d228cd9c4073), CHANGELOG.md, frontier/ledger.tsv, PR #1176

**What was asked:** Verify and record the physical i9-10900K `v0.5.15` `bench-crossover` bundle (`origin/evidence` `45ae3ff3`) per `docs/RELEASING.md §5a`, and append the `frontier/ledger.tsv` row for newly merged software PR `#1176` (`9001629a`) so `ledger.py --missing` stays clean.
**What I found:**
- Bundle integrity & parity: `crossover/run-20261127T141422Z.tar.gz` verified exact SHA-256 `cf23c1484b84c40b3c24f6b3ca877f5a2219aa5bdc98dd3e7b90d228cd9c4073` at `commit.txt = aaaee363b941a7ab64493be22e75aaeb510be861` (`v0.5.15`), 400/400 runs `termination == "success"` with identical output hash `a8830caee5c511a6` (Wikidata 210 MB, 10 rounds).
- Verdict (`docs/RELEASING.md §5a`): **Unmoved (the team threshold stays).** No forced arm beats the shipped decoder at any count where it changes the plan: `mibz2-workers-w` is `1.010 [0.973, 1.049]` at 4T (CPU `1.077`), `1.282 [1.237, 1.329]` at 6T, `1.042 [1.022, 1.062]` at 8T, `1.056 [1.031, 1.081]` at 10T, and `0.996–1.122` wall (`1.308–1.474` CPU) across SMT 12/16/20T; `mibz2-team2` is `1.206–4.533` wall (`1.102–9.708` CPU); `mibz2-team3` is `1.393–2.948` wall (`1.124–4.888` CPU).
- Sanity (same plan as shipped at 5, 7, 9 threads, verified in `observed_threads`): 7T (`0.984 [0.964, 1.005]`, CPU `0.988`) and 9T (`0.995 [0.968, 1.023]`, CPU `0.995`) span 1; 5T wall is `0.964 [0.937, 0.993]` (near-miss noise on identical 5-worker plan; CPU `0.991 [0.982, 1.001]` spans 1).
**What changed:** updated `CHANGELOG.md` `## [0.5.15]` `The team threshold (RELEASING §5a)` with the verified bundle and verdict, appended row `1176` (`default`) to `frontier/ledger.tsv`, and added this append-only delta.
**What's next:** `v0.5.15` post-release physical i9 verification (`release-scaling` + `crossover`) is 100% complete and `frontier/ledger.tsv` covers all merged software PRs through `#1176`.
**Caveats:** `bench-crossover` runs without pre-spin on single-copy Wikidata (~60–130 ms); 5T near-miss wall CI (`0.937–0.993`) is on the exact same 5-worker thread set as shipped (`CPU 0.991 [0.982, 1.001]`) and is weighed as noise under §5a.
