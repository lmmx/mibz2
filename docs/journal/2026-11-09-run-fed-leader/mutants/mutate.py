"""Apply one D049 mutant to the worktree (exact text replacement, once), write its patch,
or restore the file. Usage: mutate.py WT apply|restore NAME OUTDIR"""
import subprocess, sys
WT, op, name, outdir = sys.argv[1:5]  # the worktree, apply or restore, the mutant, where its patch goes
RT = "src/parallel/run_team.rs"
MTF = "src/mtf.rs"
M = {
 # B6: every surplus-index guard removed at once
 "B6a-owns-always": (RT, "    fn owns(&self, j: usize) -> bool {\n        j < self.shares\n    }",
                         "    fn owns(&self, _j: usize) -> bool {\n        true\n    }"),
 # B6: only count's guard removed (D049's witness: hist.add(64) before any check)
 "B6b-count-unguarded": (RT, "    fn count(&self, j: usize) {\n        if !self.owns(j) {\n            return;\n        }\n",
                             "    fn count(&self, j: usize) {\n"),
 # B6: count's guard and records_of's debug assertion both removed (the tests' own
 # detection, without the assertion, which a release build doesn't have)
 "B6c-count-unguarded-no-assert": (RT, [("    fn count(&self, j: usize) {\n        if !self.owns(j) {\n            return;\n        }\n",
                                         "    fn count(&self, j: usize) {\n"),
                                        ("        debug_assert!(self.owns(j));\n        if self.direct {", "        if self.direct {")], None),
 # R3: the shares' prefix refusing the empty tiling again (the fuzz case 596's cause)
 "R3a-empty-tiling-refused": (RT, "    hists: &mut [[[u32; 256]; 3]],\n) -> bool {\n    if recs.is_empty() || n >= 1 << 24 {",
                                  "    hists: &mut [[[u32; 256]; 3]],\n) -> bool {\n    if recs.len() < 2 || n >= 1 << 24 {"),
 # M12's twin: the current and next blocks not published in the outer Err
 "M12r-current-next": (RT, "                for b in held.into_iter().flatten() {",
                           "                for b in held.into_iter().flatten().take(0) {"),
 # the held prior (inverted, RLE1 not run) not published in the outer Err
 "M12r-held-prior": (RT, "                if let Some(b) = l.inverted.take() {\n                    let kind = ErrorKind::Internal(msg.clone());",
                         "                if let Some(b) = l.inverted.take().filter(|_| false) {\n                    let kind = ErrorKind::Internal(msg.clone());"),
 # C29: a run split at a checkpoint, the checkpoint taken before the part's bytes are counted
 "C29a-checkpoint-before-part": (MTF, "                let part = next - n;\n                run_store!(part);\n                left -= part;\n                checkpoints.push(*counts);",
                                      "                let part = next - n;\n                checkpoints.push(*counts);\n                run_store!(part);\n                left -= part;"),
 # C29: the last checkpoint of a block ending exactly at one dropped
 "C29b-final-checkpoint": (MTF, "    if result.is_ok() && n == next {\n        checkpoints.push(*counts);\n    }\n    // SAFETY: the first n bytes were written above, n <= max <= capacity\n    ll.set_len(n);\n    result.map(|()| n)\n}\n",
                              "    // SAFETY: the first n bytes were written above, n <= max <= capacity\n    ll.set_len(n);\n    result.map(|()| n)\n}\n"),
 # C29: the 64-byte vector run store taken up to 64 bytes past the block's bound
 "C29c-vector-store-bound": (MTF, "            if len <= 64 && n + 64 <= max {\n                let v = _mm256_broadcastb_epi8(f0);\n                _mm256_storeu_si256(out.add(n) as *mut __m256i, v);\n                _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);\n            } else {\n                std::ptr::write_bytes(out.add(n), c, len);\n            }\n            n += len;\n        }};\n    }\n    let mut next = CHECKPOINT;\n    let mut syms = b.syms.iter();\n    'block: loop {",
                                "            if len <= 64 && n < max {\n                let v = _mm256_broadcastb_epi8(f0);\n                _mm256_storeu_si256(out.add(n) as *mut __m256i, v);\n                _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);\n            } else {\n                std::ptr::write_bytes(out.add(n), c, len);\n            }\n            n += len;\n        }};\n    }\n    let mut next = CHECKPOINT;\n    let mut syms = b.syms.iter();\n    'block: loop {"),
}
path, old, new = M[name]
pairs = old if isinstance(old, list) else [(old, new)]
p = f"{WT}/{path}"
if op == "apply":
    s = open(p).read()
    # (C29's mutants: only in mtf_checkpoints_reg_avx2, the matched control's kernel)
    mark = "unsafe fn mtf_checkpoints_reg_avx2(" if path == MTF else ""
    head, tail = s.split(mark, 1) if mark else ("", s)
    for old, new in pairs:
        assert tail.count(old) == 1, (name, tail.count(old))
        tail = tail.replace(old, new)
    open(p, "w").write(head + mark + tail)
    d = subprocess.run(["git", "-C", WT, "diff", "--", path], capture_output=True, text=True).stdout
    open(f"{outdir}/{name}.patch", "w").write(d)
    print(f"applied {name}")
else:
    subprocess.run(["git", "-C", WT, "checkout", "--", path], check=True)
    print(f"restored {path}")
