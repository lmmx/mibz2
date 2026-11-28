claim: crc32-avx2-ring-rescue
by: claude-code
outcome: partial
evidence: PR #1295 at 30886324 (rebased onto master 31c35662 after the VOID first attempt, evidence 0390e19f); CI bench runs 46122431783 and 46102525201

**What was asked:** record CI's paired no-regression result on the re-pinned candidate before any i9 data, so §4 of docs/journal/2026-11-28-crc32-ring-rescue/registration.md is applied to it as frozen.
**What I found:** against base 31c35662, wall is slower beyond noise on primaries: wikidata/1 +1.0% (+0.7, +1.3), text/1 +0.9% (+0.5, +1.2), text/2 +3.7% (+0.7, +6.8). That fails registration §4's CI merge condition (PERFORMANCE-POLICY §2 rule 2) as it stands. The CRC stage is −0.4% (−0.9, +0.1). On the old base (run 46102525201, same CPU model), the same kernel showed no primary loss. I won't re-run CI to get a pass. The i9 run still goes ahead, because only it can measure the Intel-side gain. On an i9 PASS, the CI loss stands as a scoped finding: the kernel is not merged as the default for every x86. A vendor-gated successor (AVX2 ring on Intel only) would need its own registration.
**What changed:** this delta only.
**What's next:** run the i9 on #1295's head and read it against §3. The merge decision then combines the i9 verdict with this CI result.
**Caveats:** one CI run per base. The difference between the two CI runs could be a code-layout interaction with the newly merged src (#1274, #1294, #1312) rather than the kernel itself; nothing here separates the two.
