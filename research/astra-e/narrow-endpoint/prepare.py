"""Build a narrow-link endpoint from the pinned W16 Open-Return sources."""
from hashlib import sha256
from pathlib import Path

HERE = Path(__file__).resolve().parent
JOURNAL = HERE.parents[2] / "docs/journal/2026-11-18-open-return"


def source(name, expected):
    data = (JOURNAL / name).read_bytes()
    if sha256(data).hexdigest() != expected:
        raise ValueError(f"unexpected input source for {name}")
    return data.decode()


def change(text, old, new):
    if text.count(old) != 1:
        raise ValueError(f"expected one occurrence of {old!r}")
    return text.replace(old, new)


constructor = source("open_return.rs", "b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1")
constructor = constructor[:constructor.index('#[path = "switches.rs"]')]
constructor += '''#[path = "../../../docs/journal/2026-11-18-open-return/endpoint.rs"]
mod endpoint;
#[path = "narrow.rs"]
mod narrow;
#[path = "../../../docs/journal/2026-11-18-open-return/segments.rs"]
mod segments;
'''
constructor = change(constructor, '    endpoint: endpoint::Endpoint,',
                     '    endpoint: endpoint::Endpoint,\n    narrow: narrow::Endpoint,')
constructor = change(constructor, '            self.induced(n, &thin, &mut stats)', '''            // H strips plus at most H+1 gap nodes on each side, plus the sentinel.
            if self.strips.len() <= (u16::MAX as usize - 3) / 3 {
                self.induced_narrow(n, &thin, &mut stats)
            } else {
                self.induced(n, &thin, &mut stats)
            }''')
(HERE / 'constructor.rs').write_text(constructor)

narrow = source('endpoint.rs', '7a9b3e8f54d5ecda262a7b670ede9b295e1d17ae245bf89af3e64a1fa6db8e29')
for old, new in [
    ('prev: [u32; 2]', 'prev: [u16; 2]'), ('next: [u32; 2]', 'next: [u16; 2]'),
    ('order: [Vec<u32>; 2]', 'order: [Vec<u16>; 2]'),
    ('terminals: [Vec<u32>; 2]', 'terminals: [Vec<u16>; 2]'),
    ('fn node(&mut self, width: u32, word: u32) -> u32', 'fn node(&mut self, width: u32, word: u32) -> u16'),
    ('let id = self.nodes.len() as u32;', 'let id = self.nodes.len() as u16;'),
    ('fn cut(&mut self, id: u32', 'fn cut(&mut self, id: u16'),
    ('fn insert(&mut self, at: u32, id: u32', 'fn insert(&mut self, at: u16, id: u16'),
    ('pub(super) fn induced(', 'pub(super) fn induced_narrow('),
    ('let e = &mut self.endpoint;', 'let e = &mut self.narrow;'),
    ('e.order[0].push(i as u32 + 1);', 'e.order[0].push(i as u16 + 1);'),
    ('e.order[1].push(i + 1);', 'e.order[1].push(i as u16 + 1);'),
]:
    narrow = change(narrow, old, new)
narrow += '\nconst _: () = assert!(std::mem::size_of::<Node>() == 16);\n'
(HERE / 'narrow.rs').write_text(narrow)
