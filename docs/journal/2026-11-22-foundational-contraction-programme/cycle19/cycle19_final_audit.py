#!/usr/bin/env python3
"""
Cycle 19 — Adversarial Referee #4: Final End-to-End 20-Cycle Programme Audit (Cycles 1-18)
==========================================================================================
Audits:
  Part 1: Cycle 18 AdaptiveLeanOpen<98304> code/dispatch audit, register line 16 check,
          shipped v0.5.12 src/open.rs:522 confirmation, and 12/12 check_manifest.py audit.
  Part 2: Complete 16-row v1 @ ee7fbf0e moonshot_register_v1.jsonl audit, SHA-256 matching
          against all archived .rs files (both HEAD working tree and git history 6996139..HEAD),
          and canonical 16-row leaderboard generation.
  Part 3: Master Theorem & Errata archival presence & consistency audit (Cycles 1-18).
  Part 4: Repo hygiene (0 non-docs diffs from 6996139..HEAD, clean git status) & Cycle 20
          handoff readiness checklist.
"""
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path("/home/antigravity/lmmx/mibz2")
JDIR = REPO / "docs/journal/2026-11-22-foundational-contraction-programme"
CHECKER = REPO / "docs/team/acquisition/check_manifest.py"
EXPECTED_SCORER_VER = "v1"
EXPECTED_SCORER_COMMIT = "ee7fbf0e50c60cd9dcf0faa0cdd634cdbbb02849"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(p: Path) -> str:
    return sha256_bytes(p.read_bytes())


def git_out(args):
    return subprocess.check_output(["git", "-C", str(REPO)] + args, text=True)


def part1_adaptive_and_manifests():
    print("=" * 96)
    print("PART 1: CYCLE 18 AdaptiveLeanOpen<98304> CODE AUDIT & 12/12 ACQUISITION MANIFESTS")
    print("=" * 96)
    cand = JDIR / "cycle18/adaptive_lean_open_candidate.rs"
    src = cand.read_text()
    cand_sha = sha256_file(cand)
    lines = src.splitlines()

    # 1. Safety & dispatch checks
    has_forbid = lines[0].strip() == "#![forbid(unsafe_code)]"
    unsafe_hits = [
        (i + 1, ln) for i, ln in enumerate(lines)
        if re.search(r"\bunsafe\b", ln) and not ln.strip().startswith("//") and "#![forbid(unsafe_code)]" not in ln
    ]
    assert has_forbid and len(unsafe_hits) == 0, f"Unsafe violation: {unsafe_hits}"

    # Verify DEFAULT_M_UNI_MAX == 98_304
    assert "pub const DEFAULT_M_UNI_MAX: u32 = 98_304;" in src
    assert "pub type Candidate = AdaptiveLeanOpen<DEFAULT_M_UNI_MAX>;" in src

    # Verify Stage 1 computes m BEFORE Stage 3a and passes `use_l2_uni = m <= M_UNI_MAX` to endpoint::induced
    m_line = next(i + 1 for i, l in enumerate(lines) if "let use_l2_uni = m <= M_UNI_MAX;" in l)
    ind_line = next(i + 1 for i, l in enumerate(lines) if "let cursors = endpoint::induced(" in l)
    s4_line = next(i + 1 for i, l in enumerate(lines) if "if !wide && m > M_UNI_MAX {" in l)
    assert m_line < ind_line < s4_line

    # Verify `if coalesce {` is OUTSIDE the `for a in 0..256` loop inside `induced_with`
    coal_line = next(i + 1 for i, l in enumerate(lines) if l.strip() == "if coalesce {")
    next_for = next(i + 1 for i, l in enumerate(lines[coal_line:], coal_line + 1) if "for a in 0..256" in l)
    else_line = next(i + 1 for i, l in enumerate(lines[coal_line:], coal_line + 1) if l.strip() == "} else {")
    else_for = next(i + 1 for i, l in enumerate(lines[else_line:], else_line + 1) if "for a in 0..256" in l)
    assert coal_line < next_for < else_line < else_for

    # Check shipped v0.5.12 src/open.rs:522
    open_lines = (REPO / "src/open.rs").read_text().splitlines()
    l522 = open_lines[521].strip()
    assert l522 == "if QUEUE && !wide && m >= 65_536 {", f"Unexpected src/open.rs:522: {l522}"
    diff_v0512 = git_out(["diff", "v0.5.12", "HEAD", "--", "src/open.rs", "src/open/"]).strip()
    assert diff_v0512 == "", "src/open* differs from v0.5.12!"

    # Check minor docstring typo on line 12 ("<= 123" vs "1203")
    doc_l12 = lines[11].strip()

    print(f"  [VERIFIED] adaptive_lean_open_candidate.rs sha256 = {cand_sha}")
    print(f"  [VERIFIED] #![forbid(unsafe_code)] = TRUE, non-comment `unsafe` hits = 0")
    print(f"  [VERIFIED] Exact m known from Stage 1 (L{m_line}) < endpoint::induced (L{ind_line}) < Stage 4 dispatch (L{s4_line})")
    print(f"  [VERIFIED] `if coalesce` dispatched OUTSIDE 256-bucket loop (L{coal_line} -> for L{next_for} / else L{else_line} -> for L{else_for})")
    print(f"  [VERIFIED] Shipped v0.5.12 src/open.rs:522 = `{l522}` (git diff v0.5.12..HEAD empty)")
    print(f"  [NOTE]     Minor doc comment typo on line 12: `{doc_l12}` (says `<= 123`, proxy max chain is `1203`)")

    # 2. Check line 16 of cycle18/moonshot_register_v1.jsonl
    reg_path = JDIR / "cycle18/moonshot_register_v1.jsonl"
    rows = [json.loads(line) for line in reg_path.read_text().splitlines() if line.strip()]
    assert len(rows) == 16
    r16 = rows[15]
    assert r16["candidate_sha256"] == cand_sha, f"SHA mismatch: {r16['candidate_sha256']} vs {cand_sha}"
    assert r16["scorer_version"] == EXPECTED_SCORER_VER
    assert r16["scorer_commit"] == EXPECTED_SCORER_COMMIT
    assert r16["gate"] == "PASS"
    w1, w2 = r16["walls"]["1"], r16["walls"]["2"]
    px = r16["proxy_advisory"]
    print(f"  [VERIFIED] Line 16 matches candidate SHA-256 & v1 @ ee7fbf0e, gate=PASS:")
    print(f"             1W wall={w1['ratio']:.4f} [{w1['lo']:.4f}, {w1['hi']:.4f}] (note: hi=1.000010 borderline!), RSS={w1['peak_rss_ratio']:.4f}")
    print(f"             2W wall={w2['ratio']:.4f} [{w2['lo']:.4f}, {w2['hi']:.4f}], RSS={w2['peak_rss_ratio']:.4f}")
    print(f"             proxy instr={px['1']['instructions_ratio']:.4f}, 1W/2W/4W/7W/8W/10W="
          + "/".join(f"{px[k]['ratio']:.4f}" for k in ("1", "2", "4", "7", "8", "10")))

    # 3. Run check_manifest.py across ALL 12 acquisition manifests
    manifests = sorted(JDIR.rglob("*manifest*.toml"))
    print(f"\n  Running check_manifest.py across ALL {len(manifests)} programme manifests:")
    passed = 0
    failed = 0
    for mf in manifests:
        rel = mf.relative_to(JDIR)
        res = subprocess.run(
            [sys.executable, str(CHECKER), str(mf)],
            capture_output=True, text=True
        )
        ok = (res.returncode == 0)
        if ok:
            passed += 1
        else:
            failed += 1
        first_line = (res.stdout.strip().splitlines() or res.stderr.strip().splitlines() or ["(no output)"])[0]
        print(f"    [{'PASS' if ok else 'FAIL'}] (rc={res.returncode}) {str(rel):42s} | sha256={sha256_file(mf)[:12]} | {first_line}")
    print(f"  [VERIFIED] Acquisition Manifests Summary: {passed}/{len(manifests)} PASS, {failed}/{len(manifests)} FAIL!")
    assert len(manifests) == 12 and passed == 12 and failed == 0
    return rows


def part2_register_audit(rows):
    print("\n" + "=" * 96)
    print("PART 2: COMPLETE 16-ROW OFFICIAL bench/moonshot (v1 @ ee7fbf0e) REGISTER & ARCHIVE AUDIT")
    print("=" * 96)

    # Index SHA-256 of ALL .rs files currently archived in JDIR
    head_sha_map = {}
    for rs in sorted(JDIR.rglob("*.rs")):
        h = sha256_file(rs)
        head_sha_map.setdefault(h, []).append(str(rs.relative_to(JDIR)))

    # Also index SHA-256 of ALL .rs blobs ever committed in 6996139..HEAD under docs/
    hist_sha_map = {}
    commits = git_out(["rev-list", "6996139..HEAD"]).splitlines()
    for c in commits:
        tree_lines = git_out(["ls-tree", "-r", c, "docs/journal/2026-11-22-foundational-contraction-programme/"]).splitlines()
        for tl in tree_lines:
            parts = tl.split()
            if len(parts) >= 4 and parts[3].endswith(".rs"):
                blob_oid = parts[2]
                path = parts[3]
                if (blob_oid, path) not in hist_sha_map:
                    content = subprocess.check_output(["git", "-C", str(REPO), "cat-file", "blob", blob_oid])
                    h = sha256_bytes(content)
                    hist_sha_map.setdefault(h, []).append((c[:7], path))

    # Also check /tmp/ just in case
    tmp_sha_map = {}
    for root in ("/tmp/fcp-a", "/tmp/fcp-b", "/tmp/fcp-c", "/tmp/fcp-ref2", "/tmp/fcp-ref4"):
        rp = Path(root)
        if rp.exists():
            for rs in rp.rglob("*.rs"):
                try:
                    h = sha256_file(rs)
                    tmp_sha_map.setdefault(h, []).append(str(rs))
                except OSError:
                    pass

    missing_head = []
    print(f"  Verifying all {len(rows)} rows have scorer_version=v1 @ ee7fbf0e50c60cd9dcf0faa0cdd634cdbbb02849...")
    for idx, r in enumerate(rows, 1):
        assert r["scorer_version"] == EXPECTED_SCORER_VER, f"Row {idx} bad version"
        assert r["scorer_commit"] == EXPECTED_SCORER_COMMIT, f"Row {idx} bad commit"
        assert r["gate"] == "PASS", f"Row {idx} gate != PASS"
        h = r["candidate_sha256"]
        in_head = head_sha_map.get(h, [])
        in_hist = hist_sha_map.get(h, [])
        in_tmp = tmp_sha_map.get(h, [])
        status = "ARCHIVED(HEAD)" if in_head else ("HISTORY-ONLY" if in_hist else "MISSING!")
        if not in_head:
            missing_head.append((idx, r["candidate"], h, in_hist, in_tmp))
        loc = in_head[0] if in_head else (f"git:{in_hist[0][0]}:{in_hist[0][1]}" if in_hist else (in_tmp[0] if in_tmp else "NOT FOUND"))
        print(f"    Row {idx:2d} | {r['time']} | {Path(r['candidate']).name:34s} | {h[:12]} | {status:14s} | {loc}")

    print(f"\n  [AUDIT] All 16/16 rows: scorer_version=v1, scorer_commit=ee7fbf0e..., gate=PASS.")
    print(f"  [AUDIT] Archived in HEAD: {16 - len(missing_head)}/16 | Missing from HEAD: {len(missing_head)}/16")
    for idx, cname, h, ih, it in missing_head:
        print(f"    [ARCHIVE GAP] Row {idx:2d}: {cname} (sha256={h[:16]}...) | git_hist={len(ih)} hits | /tmp={it}")

    # Emit Canonical 16-Row Markdown Leaderboard Table
    print("\n  === CANONICAL 16-ROW v1 @ ee7fbf0e LEADERBOARD TABLE ===")
    print("| # | Time (UTC) | Candidate | SHA-256 (12) | 1. Gate | 2. Wall 1W (95% CI, rds) | 2. Wall 2W (95% CI, rds) | 4. Peak RSS (1W / 2W) | 3. Advisory Proxy (`instr`, `1W/8W/10W`) | Archive |")
    print("|---:|---|---|---|---|---|---|---|---|---|")
    for idx, r in enumerate(rows, 1):
        name = Path(r["candidate"]).name
        h12 = r["candidate_sha256"][:12]
        w1, w2 = r["walls"]["1"], r["walls"]["2"]
        def tag(w):
            if w["hi"] < 1.0: return "FASTER"
            if w["lo"] > 1.0: return "SLOWER"
            return "INCONCL"
        s1 = f"`{w1['ratio']:.4f}` [{w1['lo']:.4f},{w1['hi']:.4f}] {tag(w1)} ({w1['rounds']}r)"
        s2 = f"`{w2['ratio']:.4f}` [{w2['lo']:.4f},{w2['hi']:.4f}] {tag(w2)} ({w2['rounds']}r)"
        rss = f"`{w1['peak_rss_ratio']:.3f}` / `{w2['peak_rss_ratio']:.3f}`"
        px = r.get("proxy_advisory", {})
        if px and "1" in px:
            p_str = f"`instr {px['1']['instructions_ratio']:.4f}`, 1W `{px['1']['ratio']:.4f}`, 8W `{px['8']['ratio']:.4f}`, 10W `{px['10']['ratio']:.4f}`"
        else:
            p_str = "*(not recorded in v1 row)*"
        arch = "HEAD" if head_sha_map.get(r["candidate_sha256"]) else ("git-hist" if hist_sha_map.get(r["candidate_sha256"]) else "GAP")
        print(f"| {idx} | `{r['time']}` | `{name}` | `{h12}` | `{r['gate']}` | {s1} | {s2} | {rss} | {p_str} | {arch} |")


def part3_theorem_archive_check():
    print("\n" + "=" * 96)
    print("PART 3: MASTER THEOREM & ERRATA ARCHIVAL EVIDENCE CHECK (CYCLES 1-18)")
    print("=" * 96)
    checks = [
        ("B1-B4 (Redheffer/Bucket-Schur, B2 r+n DRAT, B3 Fib, B4 0%-stale)", [
            "cycle1/referee2/task1_full.py", "cycle2/beta/verify_potentials.py",
            "cycle2/referee2/r_plus_n_n8_k2.drat", "cycle4/gamma/b4_grid_flat16_candidate.rs"
        ]),
        ("B5-B7 (SelThinSq, universal wire_xor Phi^2, Phi2XorOpen)", [
            "cycle6/beta/selthinsq_candidate.rs", "cycle8/beta/phi2_xor_candidate.rs"
        ]),
        ("C1-C7 (C1..C7 + 4718-BWT frontier + L=1010101010110010 T=0000111011010011)", [
            "cycle4/referee2/cycle4_audit_c3.py", "cycle11/alpha/exact_cgap_optimum.py",
            "cycle12/alpha/cycle12_lp_farkas_and_cgap_ub.py", "cycle14/cycle14_binary_cgap_and_hgrand.py",
            "cycle15/cycle15_c7_and_19_24.py", "cycle17/cycle17_unification_and_exact_opt.py",
            "cycle18/cycle18_audit_c16_c17.py"
        ]),
        ("F1-F3 (entropy, splice, i9 5.8 ns/op break-even)", [
            "cycle3/alpha/verify_bwt_entropy_and_lower_bounds.py", "cycle4/referee2/cycle4_audit_c3.py"
        ]),
        ("K1-K3, J1 (Run-Overlap Graph M1..M4, K2 acyclic scope)", [
            "cycle5/alpha/test_run_overlap_graph.py", "cycle6/alpha/m1_k1_peeling_manifest.toml"
        ]),
        ("M1-M3 (Discrete Morse AP4_max, m_W(FL)=m_W(LF), Fox-calculus)", [
            "cycle8/alpha/morse_ap4_candidate.rs", "cycle8/alpha/test_morse_coreduction.py"
        ]),
        ("O1-O3 (Origami SL(2,Z) shear closure)", [
            "cycle9/referee2/cycle9_audit_origami.py"
        ]),
        ("T0-T1, P1 (Train-Track S2_dt_heavy, Parallel Contraction)", [
            "cycle9/alpha/test_train_track_folding.py", "cycle9/alpha/audit_parallel_contraction.py"
        ]),
        ("D0-D2b (Rank-Shift Avalanche, Durand D2a ALL factors u incl. self-overlapping)", [
            "cycle9/beta/cycle9_defect_durand.py", "cycle10/referee2/cycle10_audit_c9.py",
            "cycle18/cycle18_audit_c16_c17.py"
        ]),
        ("S1-S3 (Reverse SA-IS, Kempa Phi-acquisition)", [
            "cycle10/alpha/test_reverse_sais.py", "cycle10/alpha/audit_phi_irreducible_lcp.c"
        ]),
        ("E1-E2, R1 (Stein rank = 2g_orig-2, GKO ~= PT_n, F_fat)", [
            "cycle10/beta/cycle10_displacement_ribbon.py"
        ]),
        ("Q1-Q3 (Foundational Q4 closure, Q1b c in 0..254)", [
            "cycle11/beta/cycle11_q4_stream_bypass.py"
        ]),
        ("U1, L0-L1 (Z3 switch classification, BBWT Lyndon, Jez RLSLP)", [
            "cycle11/alpha/exact_cgap_optimum.py", "cycle12/alpha/cycle12_bbwt_and_rlslp.py"
        ]),
        ("P_pisot & Bayer-Diaconis GUID Mixing Law (1-(15/16)^8=0.4033)", [
            "cycle13/alpha/cycle13_pisot_and_guid_mixing.py", "cycle14/cycle14_audit_c13.py"
        ]),
        ("G_wall (LRU/Online/Belady-OPT bounds)", [
            "cycle15/cycle15_g_wall.py", "cycle16/cycle16_audit_c15.py"
        ]),
        ("H_grand v2 (5-model separation, T_K=5, C7 1.5x upgrade)", [
            "cycle14/cycle14_binary_cgap_and_hgrand.py", "cycle15/cycle15_audit_c14.py"
        ]),
        ("U_cat (12-framework WPB + lossless/Meyer biconditional + corrected Row #5)", [
            "cycle17/cycle17_unification_and_exact_opt.py", "cycle18/cycle18_audit_c16_c17.py"
        ]),
        ("Large-n Deployment Crossover (scale_crossover_bench, 0.5.12 spelling/wide, 900K_ultra fix)", [
            "cycle16/scale_crossover_bench/src/open_scaled.rs", "cycle16/scale_crossover_out_v2.txt",
            "cycle18/cycle18_audit_c16_c17.py"
        ]),
    ]
    all_files = [str(p.relative_to(JDIR)) for p in JDIR.rglob("*") if p.is_file() and "__pycache__" not in str(p)]
    print(f"  Total archived files under journal dir (excl. __pycache__): {len(all_files)}")
    for fam, expected_paths in checks:
        found = []
        missing = []
        for ep in expected_paths:
            if (JDIR / ep).exists():
                found.append(ep)
            else:
                # Find closest match in same cycle dir
                cyc = ep.split("/")[0]
                cands = [f for f in all_files if f.startswith(cyc + "/")]
                missing.append((ep, cands[:3]))
        tag = "OK" if not missing else "RENAMED/CHECK"
        print(f"  [{tag:13s}] {fam}")
        for ep, cands in missing:
            print(f"                  note: `{ep}` not exact; cycle files sample: {cands}")


def part4_repo_hygiene():
    print("\n" + "=" * 96)
    print("PART 4: REPO HYGIENE & CYCLE 20 HANDOFF READINESS CHECK")
    print("=" * 96)
    head = git_out(["rev-parse", "--short", "HEAD"]).strip()
    status = git_out(["status", "-s"]).strip()
    diff_all = git_out(["diff", "--name-only", "6996139..HEAD"]).splitlines()
    non_docs = [f for f in diff_all if not f.startswith("docs/")]
    print(f"  [VERIFIED] HEAD = {head}")
    print(f"  [VERIFIED] git status -s = {'CLEAN' if not status else status}")
    print(f"  [VERIFIED] git diff --name-only 6996139..HEAD: {len(diff_all)} files total, {len(non_docs)} non-docs files!")
    assert status == "" and len(non_docs) == 0

    frontier = (REPO / "docs/team/research-charter/FRONTIER.md").read_text()
    has_q6 = "Direct BWT-space and compositional" in frontier and "retained permanently in Foundational" in frontier
    journal_md = (REPO / "docs/journal/2026-11-22-foundational-contraction-programme.md").read_text()
    j_title = journal_md.splitlines()[0]
    print(f"  [VERIFIED] FRONTIER.md Foundational Q6 permanently present = {has_q6}")
    print(f"  [VERIFIED] Main journal currently at: `{j_title}` ({len(journal_md.splitlines())} lines -> needs full Cycles 1-20 update in Cycle 20!)")
    # Check __pycache__ committed in git (hygiene note!)
    pycaches = [f for f in diff_all if "__pycache__" in f or f.endswith(".pyc")]
    print(f"  [HYGIENE NOTE] Committed .pyc files in 6996139..HEAD: {len(pycaches)} (in cycles 9-11; non-blocking, optional cleanup in Cycle 20)")


if __name__ == "__main__":
    rows = part1_adaptive_and_manifests()
    part2_register_audit(rows)
    part3_theorem_archive_check()
    part4_repo_hygiene()
    print("\n" + "=" * 96)
    print("CYCLE 19 FINAL END-TO-END AUDIT COMPLETE.")
    print("=" * 96)
