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
    top_bands: Vec<Band>,
    offsets: Vec<u32>,
    spelling: Vec<u8>,
}
#[derive(Clone, Copy)]
struct AnchorRun {
    start: u32,
    end: u32,
    base: u32,
}
#[derive(Clone, Copy)]
struct Macro {
    delta: u32,
    code: u32,
    tail_end: u32,
}
#[derive(Default)]
struct Work {
    thin_runs: usize,

    splits: usize,
    lookups: usize,

    right_min: usize,
    max_tail: usize,
    emit_steps: usize,
    physical: usize,
    orbit_area: usize,
}
#[derive(Default)]
struct ResultMap {
    rows: Index,
    payload: Vec<u32>,
    extra: Vec<u32>,
    bits: u32,
    mask: u32,
    commands: Index,
    starts: Vec<AnchorRun>,
    macros: Vec<Macro>,
    tail: Vec<u8>,
    m: u32,
    anchors: u32,
    mark: u32,
    work: Work,
}
const MARK: u32 = 1 << 31;

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
    fn factor<const AUDIT: bool>(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        map: &mut ResultMap,
    ) {
        let (mut all, mut thin, mut thick) = ([0; 256], [0; 256], [0; 256]);
        map.reset();
        let (mut mark, mut m) = (0, 0);
        self.strips.clear();
        runs_of(records, p, limit, |byte, x, w, keep| {
            let c = byte as usize;
            if keep {
                if (x..x + w).contains(&p) {
                    mark = m + p - x;
                }
                thin[c] += w;
                if AUDIT {
                    map.work.thin_runs += 1;
                }
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

        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        map.m = m;
        map.mark = mark;
        self.induced(n, &thin, map);
        map.bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        map.mask = (1 << map.bits) - 1;
        let wide = (254 + self.offsets.len() as u32) > u32::MAX >> map.bits;
        map.rows.size(m);
        chunks(
            records,
            p,
            limit,
            &self.top_bands,
            bases,
            |x, w, _j, code| {
                map.rows.mark(x);
                if code < 256 && (x..x + w).contains(&mark) {
                    map.rows.mark(mark);
                    if mark + 1 < x + w {
                        map.rows.mark(mark + 1);
                    }
                }
                if AUDIT {
                    map.work.splits += 1;
                }
            },
        );
        let count = map.rows.seal(m);
        map.payload.resize(count as usize, 0);
        map.extra.clear();
        if wide {
            map.extra.resize(count as usize, 0);
        }
        chunks(
            records,
            p,
            limit,
            &self.top_bands,
            bases,
            |x, w, j, code| {
                let delta = j.wrapping_sub(x) & map.mask;
                let mut entry = map.rows.at(x) as usize;
                let mut store = || {
                    map.payload[entry] = delta | if wide { 0 } else { code << map.bits };
                    if wide {
                        map.extra[entry] = code;
                    }
                    entry += 1;
                };
                store();
                if code < 256 && (x..x + w).contains(&mark) {
                    if x < mark {
                        store();
                    }
                    if mark + 1 < x + w {
                        store();
                    }
                }
            },
        );
        map.compile::<AUDIT>();
    }
}

impl ResultMap {
    fn compact(&self, x: u32) -> u32 {
        let a = self.starts[self.starts.partition_point(|a| a.end <= x)];
        assert!(x >= a.start);
        a.base + x - a.start
    }

    fn compile<const AUDIT: bool>(&mut self) {
        self.commands.size(self.anchors);
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
                    if AUDIT {
                        self.work.lookups += 1;
                    }
                    let (id, end) = self.rows.get(y);
                    let value = self.payload[id as usize];
                    let mut code = if self.extra.is_empty() {
                        value >> self.bits
                    } else {
                        self.extra[id as usize]
                    };
                    if y == self.mark && code < 256 {
                        code |= MARK;
                    }

                    width = width.min(end - y);
                    if first.is_some() && code >= 256 {
                        break;
                    }
                    if first.is_none() {
                        first = Some(code & !MARK);
                    } else {
                        debug_assert!(code < 256);
                        self.tail.push(code as u8);
                    }
                    y = (y + (value & self.mask)) & self.mask;
                }
                let to = self.compact(y);
                self.commands.mark(compact_from);
                if AUDIT {
                    self.work.right_min += usize::from(width == 1);
                }
                if AUDIT {
                    self.work.max_tail = self.work.max_tail.max(self.tail.len() - initial_tail);
                }
                if AUDIT {
                    self.work.physical += (width as usize) * (1 + self.tail.len() - initial_tail);
                }
                self.macros.push(Macro {
                    delta: to.wrapping_sub(compact_from),
                    code: first.unwrap(),
                    tail_end: self.tail.len() as u32,
                });
                compact_from += width;
                from += width;
            }
        }
        assert_eq!(compact_from, self.anchors);
        self.commands.seal(self.anchors);
    }

    fn emit<const AUDIT: bool>(&mut self, k: &Kernel, n: usize, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(n);
        let primary = self.compact(self.mark);
        let mut row = primary;
        loop {
            let id = self.commands.at(row) as usize;
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
            if AUDIT {
                self.work.emit_steps += 1;
            }
            if row == primary {
                break;
            }
            assert!(out.len() < n);
        }
        if AUDIT {
            self.work.orbit_area = out.len();
        }
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }
}
