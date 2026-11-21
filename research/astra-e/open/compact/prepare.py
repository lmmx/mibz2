"""Keep the frozen F-coordinate algorithm and replace only its retained-run fill."""
from hashlib import sha256
from pathlib import Path

here = Path(__file__).resolve().parent
source = (here.parent / "candidate.rs").read_bytes()
assert sha256(source).hexdigest() == "bed3163e978ec41d387da33f0a18b0af5161f5bc4029b0331879ff6a3687ff0f"
s = source.decode().replace('mod endpoint;', '#[path = "../endpoint.rs"]\nmod endpoint;')
s = s.replace("    strips: Vec<Strip>,","    strips: Vec<Strip>,\n    thin_runs: Vec<u16>,")
s = s.replace("        self.strips.clear();","        self.strips.clear();\n        self.thin_runs.clear();")
s = s.replace("                thin[a] += w;","                self.thin_runs.push((u16::from(byte) << 8) | w as u16);\n                thin[a] += w;")
s = s.replace(", LIMIT>(records, p, buckets, bits)", ">(buckets, bits)")
s = s[:s.index("    // A band supplies")] + '}\ninclude!("fill.rs");\n' + s[s.index("fn emit("):]
(here / "candidate.rs").write_text(s)
