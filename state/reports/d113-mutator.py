"""Mutation check of src/cells_owned.rs, the one-scan writer.

Run from the repository root: python3 docs/journal/2026-11-18-one-scan-writer/mutate.py. Each
mutation is applied alone, and the writer's tests run (cells::tests::owned_writer* and
miri_cells_owned_writer, natively). A mutation the tests do not catch is a miss.
"""
import subprocess, sys, os
SRC = "src/cells_owned.rs"
MUTATIONS = [
    ("the survival certificate inverted", "} else if w == later {", "} else if w != later {"),
    ("no wrap on the marked spine", "                if right >= p {\n                    right -= p;\n                }", "                if right >= p + p {\n                    right -= p;\n                }"),
    ("the commands run last parent first", "        for i in tail..nr {", "        for i in (tail..nr).rev() {"),
    ("no command stored", "                        if tail > j {", "                        if tail > j + nr {"),
    ("an owned child's position not set", "                        *pos.add((r - ro) / 16) = d;", "                        let _ = (r, d);"),
    ("the root one place off", "*pos.add(nr - 1) = ((p - start) % p) as u32;", "*pos.add(nr - 1) = ((p - start + 1) % p) as u32;"),
    ("the carried descriptors not updated", "            (na, nb) = (a, b);", "            let _ = (na, nb);"),
    ("the guard admits a shorter cycle", "if p != n || nr == 0 || word as usize != self.rule_end - 16 {", "if p > n || nr == 0 || word as usize != self.rule_end - 16 {"),
]
orig = open(SRC).read()
misses = []
try:
    for name, old, new in MUTATIONS:
        assert orig.count(old) == 1, name
        open(SRC, "w").write(orig.replace(old, new))
        r = subprocess.run(["rustup", "run", "1.99.1", "cargo", "test", "-q", "--release", "--lib", "owned_writer"],
                           capture_output=True, text=True, env=dict(os.environ))
        r2 = subprocess.run(["rustup", "run", "1.99.1", "cargo", "test", "-q", "--release", "--lib", "miri_cells_owned"],
                            capture_output=True, text=True, env=dict(os.environ))
        caught = r.returncode != 0 or r2.returncode != 0
        print(f"{'caught' if caught else 'MISSED'}: {name}", flush=True)
        if not caught:
            misses.append(name)
finally:
    open(SRC, "w").write(orig)
print(f"{len(MUTATIONS) - len(misses)} of {len(MUTATIONS)} mutations caught")
sys.exit(1 if misses else 0)

