# STEPS revision 4: PROCEED for the bounded diagnostic

Inbox: 6763771662. Reviewed `b23897e89a292221836685eab0f1c8ce944802d2`, `docs/journal/2026-11-10-steps-placement-diagnostic-DRAFT.md` and its pinned counter, tests and historical count files. Instructions: `docs/team/SPEC.md` and `SPEC.RESEARCHER.md` at `a2da49638ac54d7763de1c167bd054f67ffdc763`, with MODE/RULES/PROTOCOL. Prior findings: Researcher 6763607539, Guardian D092 (6763626030).

**PROCEED as a research design.** Revision 4 resolves my material objections to revision 3. Accept all four requested choices: loads as the population unit; whole D←H/4096 class contrast for c_diag/E2; overlapping loads included in outside; no PMU collection. Guardian admission and the registered implementation's correctness/safety checks remain. This is no execution, production qualification or reopening of #324's frozen verdict.

## 1. Counter repair is independently reproduced

Exact SHA-256s match the registration:

- `agg3.c`: `a0e6b030d4bd413140ae2edc1b0309da8b64618126269d3a6a0a0d932d190a63`.
- `test_agg3.sh`: `4949ad570621d537fe41093c0e4e339a8f603544286c76d4977dfdac3c1dc7a0`.
- Previous `agg3.c`, at revision 3: `1ac18c5cc33def89ebb890cc993ae5cada81e833f3f4e9bffd64ea98521f9ca0`.

Compiled the trace parser only. All **15 supplied tests pass**. Independently reconstructed the statistic using Python sets and a chronological 48-store deque, rather than the implementation's hit flags and pair hash table. On **8 seeded traces × 1,205 events**, every regional load total, class count, any-class count, named/outside/both count, pair load count, pair weight and explicit named-pair zero matches the oracle. The complete pair sets match; the completion marker is present; exits are zero and stderr empty. The same traces produce **byte-identical class/load and legacy PAIRS rows** on the old tool.

The oracle includes repeated same-PC stores; multiple named and outside PCs on one load; absent named pairs; all three regions; both moduli; partial true overlaps; M events as load-before-store; and ring eviction. This directly repairs the prior two-store, omitted-sixth-pair and overlapping-populations counterexamples. Test scope is finite valid lackey-format traces within the parser's stated low-PC domain and ordinary widths; it is not a general malformed-input/parser-hardening proof.

`class = named + outside − both` is the correct inclusion–exclusion identity. `outside = class − named` is not. Pair loads are useful individual counters but still cannot be summed to obtain the named union. The new direct union counters supply the registered gates without that substitution. END certifies parser completion, not successful upstream decoding: the registration correctly requires lackey's exit, agg3's exit, stream identity/counts and output checks as separate evidence. The eventual supervisor must preserve these statuses.

## 2. The reference and the changed gate are now coherent

Read the archived `sc-master-rand-0.txt` and `sc-head-rand-0.txt` at the reviewed commit. Their `mtf.mtf_avx2` / `mod 4K` / `D<-H` **class** fields are exactly **13,829 and 56,154**. Thus `(56,154−13,829)/890,000 = 0.0475561797752809` loads/output-byte. The frozen reference 0.0476 is its upward-rounded value; E2 uses the exact integer threshold **380,800** over 8,000,000 bytes. No pair-weight conversion or historical re-trace is needed. The dated correction preserves the old text and verdict, explicitly withdrawing the false pair-count description.

Accept c_diag on the class contrast K, not the named union N. It is a substantive prospective definition change, now explicit, and aligns the diagnostic's count unit with the only archived historical reference. It does **not** establish transport of marginal cost across binaries, hosts, first-block/whole-file composition, or lackey/native execution. Revision 4 states those limits beside the budget comparison.

**A useful bound on the new gate.** Write B for the `both` count and C=N_lo−N_hi. Inclusion–exclusion gives

`K_lo−K_hi = C + (O_lo−O_hi) − (B_lo−B_hi)`.

E1 gives `N_hi ≤ C/9`; S1 gives `|O_lo−O_hi| ≤ C`; and `B_hi ≤ N_hi`, `B_lo ≥ 0`. Consequently

`K_lo−K_hi ≤ 2C + N_hi ≤ 19C/9`.

E2 therefore implies **C ≥ ceil(9×380,800/19) = 180,379 named loads**. With the intended hi-side disjoint named geometry, N_hi=0 and the stronger bound is **C ≥ 190,400**. These follow from the existing gates; they add no new threshold. Exhaustive small-integer inclusion–exclusion checks cover 178,965 combinations in the attached probe.

So switching E2 to the class does not admit an arbitrarily tiny named contrast carried by outside counts alone. It also does not prove named dominance: for example, N_lo=10, N_hi=1, O_lo=9, O_hi=0, and B_lo=B_hi=0 satisfy E1/S1 with C=9 and class contrast 18. Counting both-population loads in outside is therefore the right conservative accounting, while S1 remains a count-specificity screen, not a bound on confounding time. The stated net-placement estimand and conditional resource-allocation rule are essential, not optional wording.

## 3. Execution objections are discharged at design level

- §6.3 now permits at most two paced retries per stage only for proven class-B failures before **any** project step; build, trace, parser/test, output-check and revalidation failures or timeouts end INVALID without restart. No pre-slot-1 loophole remains in the operative policy.
- §6.2 no longer claims that hashes and named residues transport all dynamic D/H counts. Every trace predicate is explicitly a stage-1 model property and native-exposure proxy. Native checks establish the particular geometry they measure.
- The manifest digest is external to the manifest, supplied with run/artifact IDs and recorded before stage 2; the manifest then binds the binary, inputs, tools and counts. This removes the self-verification circle.
- No PMU collection, no extra slots, fixed order/deadlines/CPU policy and no precision reruns are unambiguous. Old wording in the historical revision-3 log describes what revision 4 superseded; §6.3 is the current policy.

The twenty-block t interval is still conditional on its stated distribution/dependence assumptions. With valid/precise data, Q≥0.51 and h≤0.10, B≥ implies L≥0.31, hence S+; the already accepted allocation rule is unchanged. A budget-not-excluded result may justify contract A's **independent master-side qualification first**, never immediate fallback adoption. A small net contrast cannot eliminate a larger hidden pair effect. D082/D086, safety obligations and the original failures stand.

## 4. Consequence and evidence limits

Expected: repair the unusable outside statistic, preserve the historical unit, and close the retry/provenance overclaims. Observed: all of those repairs hold in source and independent synthetic replay. **No performance gap was measured or closed.** What advanced is the ability to evaluate the proposed intervention coherently.

The next value is the bounded diagnostic's discriminating information: whether this fixed master-derived placement produces a net contrast worth spending a separate contract-qualification experiment on. Additional counter-repair iterations or a historical re-trace are not justified by the resolved objections. Proceed through Guardian admission and implementation/freeze checks; do not broaden the experiment or infer causal attribution from its outcome.

Only the parser and synthetic/arithmetic probes ran. No decoder build/run, timing, workflow dispatch, production implementation or merge. Other items QUEUED on #145 were not taken as inbox assignments.

## Reproduce

The adjacent C and shell files are unmodified reviewed evidence copies, not a proposed implementation. From this report directory:

```sh
gcc -O2 -Wall -Wextra agg3.c -o agg3
gcc -O2 agg3-old.c -o agg3-old
bash test_agg3.sh
python probes.py
```

`history.json` holds the two exact retrieved historical outputs; `probes.json` records the replay. Routing audit since the previous publication passed; REVIEW destination is #145. Migration source head and manifest digest were checked before the receipt.
