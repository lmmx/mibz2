"""Install the spelling-free selector in the frozen directly induced word decoder."""
from pathlib import Path
d = Path(__file__).resolve().parent
source = (d.parent / "f2-word/candidate.rs").read_text()
source = source.replace("struct Candidate {", "struct Candidate {\n    heap: std::collections::BinaryHeap<(u32, std::cmp::Reverse<u32>)>,")
source = source.replace("    #[inline]\n    fn locate(", (d / "choose.rs").read_text() + "\n    #[inline]\n    fn locate(")
source = source.replace("self.for_pattern::<FORCE>(caps, n, p, out, PATTERN)",
                        'self.for_depth::<FORCE>(caps, n, p, out, b"\\\"", 2)')
old = "        if n == 0 || n > 900000"
insert = """        self.for_depth::<FORCE>(caps, n, p, out, pattern, 0)
    }

    fn for_depth<const FORCE: bool>(
        &mut self, caps: &[u16], n: usize, p: u32, out: &mut Vec<u8>,
        pattern: &[u8], levels: usize,
    ) -> bool {
"""
assert source.count(old) == 1
source = source.replace(old, insert + old)
source = source.replace("let Some((quote, tokens))", "let Some((mut quote, mut tokens))")
source = source.replace("        let Some((row, skip))", """        for _ in 0..levels {
            (quote, tokens) = self.heaviest(quote, tokens);
        }
        let Some((row, skip))""")
source = source.replace('#[path = "checks.rs"]', '#[path = "../f2-word/checks.rs"]')
source = "\n".join(line for line in source.split("\n") if not line.startswith("const PATTERN:"))
source = source.replace("    fn for_pattern", "    #[cfg(test)]\n    fn for_pattern")
source += '\n#[cfg(test)]\n#[path = "nested_checks.rs"]\nmod nested_checks;\n'
(d / "candidate.rs").write_text(source)
