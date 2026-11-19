//! First return on sparse residual translations; verifier and work ledger.
use mibz2::{
    bwt::{InverseBwt, Plain},
    runs,
    stream::{Event, Scanner, SliceSource},
    Kernels,
};
use std::{env, fs};

#[path = "../../open/endpoint.rs"]
mod endpoint;

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}
#[derive(Clone, Copy)]
struct Band {
    end: u32,
    source: u32,
    code: u32,
}
#[derive(Default)]
struct Kernel {
    endpoint: endpoint::Endpoint,
    strips: Vec<Strip>,
    image_order: Vec<u32>,
    bands: Vec<Band>,
    top_bands: Vec<Band>,
    offsets: Vec<u32>,
    spelling: Vec<u8>,
}
#[derive(Clone, Copy)]
struct Select {
    end: u32,
    from: u32,
}
#[derive(Clone, Copy)]
struct Tile {
    end: u32,
    delta: u32,
    code: u32,
}
#[derive(Clone, Copy)]
struct AnchorRun {
    start: u32,
    end: u32,
    base: u32,
}
#[derive(Clone, Copy)]
struct Macro {
    end: u32,
    delta: u32,
    code: u32,
    tail_end: u32,
}
#[derive(Default)]
struct Work {
    thin_runs: usize,
    select_tests: usize,
    splits: usize,
    lookups: usize,
    line_tests: usize,
    right_min: usize,
    max_tail: usize,
    emit_steps: usize,
    physical: usize,
    orbit_area: usize,
}
#[derive(Default)]
struct ResultMap {
    tiles: Vec<Tile>,
    starts: Vec<AnchorRun>,
    directory: Vec<u32>,
    macros: Vec<Macro>,
    tail: Vec<u8>,
    m: u32,
    anchors: u32,
    mark: u32,
    work: Work,
}
const MARK: u32 = 1 << 31;
const PAGE: u32 = 8;

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut x = 0;
    counts.map(|w| {
        let y = x;
        x += w;
        y
    })
}
fn runs_of(records: &[u16], p: u32, limit: u32, mut visit: impl FnMut(u8, u32, u32, bool)) {
    let (mut i, mut x) = (0, 0);
    while i < records.len() {
        let c = (records[i] >> 8) as u8;
        let mut w = 0;
        while i < records.len() && (records[i] >> 8) as u8 == c {
            w += u32::from(records[i] & 255);
            i += 1;
        }
        if w > limit && (x..x + w).contains(&p) {
            if x < p {
                visit(c, x, p - x, false);
            }
            visit(c, p, 1, true);
            if p + 1 < x + w {
                visit(c, p + 1, x + w - p - 1, false);
            }
        } else {
            visit(c, x, w, w <= limit);
        }
        x += w;
    }
}
impl Kernel {
    fn factor(&mut self, records: &[u16], n: u32, p: u32, limit: u32) -> ResultMap {
        let (mut all, mut thin, mut thick) = ([0; 256], [0; 256], [0; 256]);
        let mut queues: [Vec<Select>; 256] = std::array::from_fn(|_| Vec::new());
        let (mut mark, mut m) = (0, 0);
        self.strips.clear();
        runs_of(records, p, limit, |byte, x, w, keep| {
            let c = byte as usize;
            if keep {
                if (x..x + w).contains(&p) {
                    mark = m + p - x;
                }
                thin[c] += w;
                queues[c].push(Select {
                    end: thin[c],
                    from: m,
                });
                m += w;
            } else {
                self.strips.push(Strip {
                    from: x,
                    to: all[c],
                    width: w,
                    byte,
                });
                thick[c] += 1;
            }
            all[c] += w;
        });
        let first = starts(all);
        let bases = starts(thin);
        let mut write = starts(thick);
        self.image_order.resize(self.strips.len(), 0);
        for (i, s) in self.strips.iter_mut().enumerate() {
            let c = s.byte as usize;
            s.to += first[c];
            self.image_order[write[c] as usize] = i as u32;
            write[c] += 1;
        }
        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        self.induced(n, &thin);
        let mut result = ResultMap {
            m,
            mark,
            ..Default::default()
        };
        result.work.thin_runs = queues.iter().map(Vec::len).sum();
        let mut bottom = 0;
        for b in &self.bands {
            let c = bases.partition_point(|&v| v <= b.source) - 1;
            let queue = &queues[c];
            let mut local = b.source - bases[c];
            let mut index = queue.partition_point(|s| {
                result.work.select_tests += 1;
                s.end <= local
            });
            while bottom < b.end {
                let run = queue[index];
                let begin = if index == 0 { 0 } else { queue[index - 1].end };
                let next = bottom + (run.end - local).min(b.end - bottom);
                let delta = (run.from + local - begin).wrapping_sub(bottom);
                result.work.splits += 1;
                // Mark a singleton even when its ordinary edge is literal.
                if b.code < 256 && (bottom..next).contains(&mark) {
                    result.tile(bottom, mark, delta, b.code);
                    result.tile(mark, mark + 1, delta, b.code | MARK);
                    result.tile(mark + 1, next, delta, b.code);
                } else {
                    result.tile(bottom, next, delta, b.code);
                }
                local += next - bottom;
                bottom = next;
                index += 1;
            }
        }
        assert_eq!(bottom, m);
        drop(queues);
        result.compile();
        result
    }
}

impl ResultMap {
    fn tile(&mut self, begin: u32, end: u32, delta: u32, code: u32) {
        if begin == end {
            return;
        }
        if let Some(prev) = self.tiles.last_mut() {
            if prev.code == code && prev.delta == delta {
                prev.end = end;
                return;
            }
        }
        self.tiles.push(Tile { end, delta, code });
    }

    fn compact(&self, x: u32) -> u32 {
        let a = self.starts[self.starts.partition_point(|a| a.end <= x)];
        assert!(x >= a.start);
        a.base + x - a.start
    }

    fn compile(&mut self) {
        let mut x = 0;
        for t in &self.tiles {
            if t.code >= 256 {
                if let Some(last) = self.starts.last_mut().filter(|a| a.end == x) {
                    last.end = t.end;
                } else {
                    self.starts.push(AnchorRun {
                        start: x,
                        end: t.end,
                        base: self.anchors,
                    });
                }
                self.anchors += t.end - x;
            }
            x = t.end;
        }
        let mut i = 0;
        for x in (0..self.m).step_by(PAGE as usize) {
            while self.tiles[i].end <= x {
                i += 1;
            }
            self.directory.push(i as u32);
        }
        let mut compact_from = 0;
        for anchor_index in 0..self.starts.len() {
            let a = self.starts[anchor_index];
            let mut from = a.start;
            while from < a.end {
                let mut y = from;
                let mut width = a.end - from;
                let mut first = None;
                let initial_tail = self.tail.len();
                loop {
                    self.work.lookups += 1;
                    let mut q = self.directory[(y / PAGE) as usize] as usize;
                    while self.tiles[q].end <= y {
                        self.work.line_tests += 1;
                        q += 1;
                    }
                    let t = self.tiles[q];
                    width = width.min(t.end - y);
                    if first.is_some() && t.code >= 256 {
                        break;
                    }
                    if first.is_none() {
                        first = Some(t.code & !MARK);
                    } else {
                        debug_assert!(t.code < 256);
                        self.tail.push(t.code as u8);
                    }
                    y = y.wrapping_add(t.delta);
                }
                let to = self.compact(y);
                self.work.right_min += usize::from(width == 1);
                self.work.max_tail = self.work.max_tail.max(self.tail.len() - initial_tail);
                self.work.physical += (width as usize) * (1 + self.tail.len() - initial_tail);
                self.macros.push(Macro {
                    end: compact_from + width,
                    delta: to.wrapping_sub(compact_from),
                    code: first.unwrap(),
                    tail_end: self.tail.len() as u32,
                });
                compact_from += width;
                from += width;
            }
        }
        assert_eq!(compact_from, self.anchors);
    }

    fn emit(&mut self, k: &Kernel, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        let primary = self.compact(self.mark);
        let mut row = primary;
        loop {
            let id = self.macros.partition_point(|m| m.end <= row);
            let band = self.macros[id];
            if band.code < 256 {
                out.push(band.code as u8);
            } else {
                let i = (band.code - 256) as usize;
                out.extend_from_slice(
                    &k.spelling[k.offsets[i] as usize..k.offsets[i + 1] as usize],
                );
            }
            let begin = if id == 0 {
                0
            } else {
                self.macros[id - 1].tail_end
            };
            out.extend_from_slice(&self.tail[begin as usize..band.tail_end as usize]);
            row = row.wrapping_add(band.delta);
            self.work.emit_steps += 1;
            if row == primary {
                break;
            }
            assert!(out.len() < n);
        }
        self.work.orbit_area = out.len();
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
        out
    }
}

fn plain(records: &[u16], p: u32) -> Vec<u8> {
    let mut l = Vec::new();
    let mut c = [0; 256];
    for &r in records {
        let b = (r >> 8) as u8;
        l.resize(l.len() + (r & 255) as usize, b);
        c[b as usize] += u32::from(r & 255);
    }
    let mut out = Vec::new();
    Plain::default().invert(&l, &c, p, &mut out);
    out
}

fn small() {
    let mut k = Kernel::default();
    let mut count = 0;
    for n in 1..=10u32 {
        for mask in 0..1 << n {
            let r: Vec<u16> = (0..n)
                .map(|i| 1 | (((mask >> i) & 1) << 8) as u16)
                .collect();
            for p in 0..n {
                let expected = plain(&r, p);
                for limit in [1, 2, 3] {
                    let mut map = k.factor(&r, n, p, limit);
                    assert_eq!(
                        map.emit(&k, n as usize),
                        expected,
                        "n={n}, mask={mask}, p={p}"
                    );
                    count += 1;
                }
            }
        }
    }
    println!("PASS {count} binary column/mark/cutoff combinations");
}

fn real(path: &str) {
    let data = fs::read(path).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut r = runs::Runs::default();
    let mut k = Kernel::default();
    let (mut ord, mut large) = (0, 0);
    println!("ordinal,n,admitted,records,m,thin_runs,first_bands,intersections,tiles,select_tests,anchor_ranges,anchors,macro_bands,width_one,total_residual_edges,lookup_count,page_extra_tests,tail_bytes,max_tail,marked_anchors,orbit_bytes,plain_equal");
    while let Event::Block { symbols, .. } = scanner.next(syms).unwrap() {
        let n = runs::front(&symbols, &mut r, Kernels::detect()).unwrap();
        if n >= 65536 && large % 24 == 0 {
            let p = symbols.orig_ptr;
            let mut map = k.factor(r.records(), n as u32, p, 16);
            let text = map.emit(&k, n);
            assert_eq!(text, plain(r.records(), p));
            let w = &map.work;
            println!(
                "{ord},{n},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},true",
                runs::takes_runs(&symbols) && r.records().len() * 20 < n * 3,
                r.records().len(),
                map.m,
                w.thin_runs,
                k.bands.len(),
                w.splits,
                map.tiles.len(),
                w.select_tests,
                map.starts.len(),
                map.anchors,
                map.macros.len(),
                w.right_min,
                w.physical,
                w.lookups,
                w.line_tests,
                map.tail.len(),
                w.max_tail,
                w.emit_steps,
                w.orbit_area
            );
        }
        large += usize::from(n >= 65536);
        ord += 1;
        syms = symbols.syms;
    }
    assert_eq!(large, 233);
}

fn main() {
    let arg = env::args().nth(1).expect("small or Wikidata corpus path");
    if arg == "small" {
        small()
    } else {
        real(&arg)
    }
}
