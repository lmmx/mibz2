"""Verify the eager test comparator and unchanged production writer against365."""

import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASE = "d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2"


def function(text, name):
    """Extract one balanced Rust function, whose bodies have balanced comments."""
    start = text.index("    fn " + name + "(")
    brace = text.index("{", start)
    depth = 1
    end = brace + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return text[start:end]


def main():
    """Assert source correspondence; print exact hashes for the evidence bundle."""
    original = subprocess.check_output(["git", "show", BASE + ":src/cells.rs"], cwd=ROOT).decode()
    candidate = (ROOT / "src/cells.rs").read_text()
    eager = (ROOT / "src/cells_original_test.rs").read_text()
    eager = function(eager, "induce_cells_original").replace(
        "induce_cells_original", "induce_cells"
    )
    eager = re.sub(
        r"#\[cfg\(test\)\]\s*self.record_materialization\(\[.*?\]\);", "", eager, flags=re.S
    )
    eager = re.sub(r"#\[cfg\(test\)\]\s*self.record_selected_slots\(ts, bs\);", "", eager)
    compact = lambda s: re.sub(r"\s+", "", s)
    assert compact(eager) == compact(function(original, "induce_cells"))
    for name in ["slots", "write_cells", "try_invert"]:
        assert function(candidate, name) == function(original, name), name
    print(
        json.dumps(
            {
                "baseline": BASE,
                "original_induction_unchanged_except_name_hooks": True,
                "writer_slots_admission_unchanged": True,
                "original_sha256": hashlib.sha256(original.encode()).hexdigest(),
                "candidate_sha256": hashlib.sha256(candidate.encode()).hexdigest(),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
