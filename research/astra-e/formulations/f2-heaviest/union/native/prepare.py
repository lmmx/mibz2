"""Build the native union constructor from the single word's stable inverse merge."""
from pathlib import Path
d=Path(__file__).resolve().parent
s=(d.parents[2]/"f2-word/candidate.rs").read_text()
s="\n".join(line for line in s.split("\n") if not line.startswith("const PATTERN:"))
s=s.replace("    fallback: Segmented,", """    fallback: Segmented,
    heap: std::collections::BinaryHeap<(u32, std::cmp::Reverse<u32>)>,
    parts: Vec<(u32, u32, u32)>,""")
i=s.index("    fn transfer_mark"); j=s.index("        self.nodes.clear();",i)
s=s[:i]+(d/"sections.rs").read_text()+"\n    fn build(&mut self) {\n"+s[j:]
i=s.index("        let mut root = Node::new(section"); j=s.index("        let mut epoch",i)
s=s[:i]+"""        for &(left, right, _) in &self.parts {
            let mut root = Node::new(left, NONE, 0);
            root.hi = right;
            self.nodes.push(root);
        }
"""+s[j:]
s=s.replace("|| (section..section + tokens).contains(&mapped)", "|| self.hit(r.byte, mapped).is_some()")
s=s.replace("if (section..section + tokens).contains(&mapped) {\n                    debug_assert!(mapped + width <= section + tokens);","if let Some(rank) = self.hit(r.byte, mapped) {")
s=s.replace("self.nodes[i].base = mapped - section;", "self.nodes[i].base = rank;")
s=s.replace("        for p in &self.merged[self.nodes[0].out as usize..] {","""        for root in 0..self.parts.len() {
            let end = if root == 0 { self.merged.len() } else { self.nodes[root-1].out as usize };
            for p in &self.merged[self.nodes[root].out as usize..end] {""")
s=s.replace("        debug_assert_eq!(input, tokens);", "        }\n        debug_assert_eq!(input, tokens);")
s=s.replace('self.for_pattern::<FORCE>(caps, n, p, out, PATTERN)','self.for_k::<FORCE>(caps, n, p, out, 8)')
s=s.replace("fn for_pattern<const FORCE: bool>","fn for_k<const FORCE: bool>").replace("        pattern: &[u8],","        k: usize,")
s=s.replace("self.prepare(caps, n as u32, pattern)",'''self.prepare(caps, n as u32, b"\\\"")''')
s=s.replace("let Some((row, skip)) = self.transfer_mark(p, quote, tokens)", "let tokens = self.select(quote, tokens, k);\n        let Some((row, skip)) = self.transfer(p)")
s=s.replace("self.build(quote, tokens);","self.build();")
s=s.replace('#[path = "checks.rs"]', '#[path = "checks.rs"]')
(d/"candidate.rs").write_text(s)
