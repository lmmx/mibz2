// The original compact source has end+byte and a wrapped LF displacement.
#[derive(Default, Clone, Copy)]
struct Range {
    lo: u32,
    hi: u32,
    flat: u32,
}

#[derive(Default, Clone, Copy)]
struct Directed {
    end_tag: u32,
    delta: u32,
}
impl Directed {
    #[inline]
    fn end(self) -> u32 {
        self.end_tag >> 9
    }
    #[inline]
    fn byte(self) -> u8 {
        self.end_tag as u8
    }
    #[inline]
    fn hit(self) -> bool {
        self.end_tag & 256 != 0
    }
    #[inline]
    fn map(self, x: u32) -> u32 {
        x.wrapping_add(self.delta)
    }
}

#[derive(Default, Clone, Copy)]
struct StageWord {
    seed: u32,
    bytes: u32,
    length: u32,
}
#[derive(Default, Clone, Copy)]
struct Frame {
    lo: u32,
    hi: u32,
    slot: u32,
    desc: u32,
    depth: u32,
}
#[derive(Default)]
struct Lane {
    word: Vec<u8>,
    pending: Vec<Frame>,
}
#[derive(Default)]
pub struct Subject {
    base: Dispatch<2, 2>,
    seeds: Vec<Range>,
    gaps: Vec<Range>,
    seams: Vec<u32>,
    hot: Vec<Range>,
    owners: Vec<Piece>,
    stage: Vec<StageWord>,
    payload: Vec<u8>,
    direction: Vec<Directed>,
    lanes: [Lane; 8],
    mark: (u32, u32),
    quote: u32,
    tokens: u32,
    spans: u32,
}
impl Subject {
    pub fn measure(
        &mut self,
        r: &Runs,
        n: u32,
        p: u32,
        threshold: u32,
        lanes: usize,
    ) -> Option<[u64; 6]> {
        self.measure_raw(r.records(), r.byte_counts(), n, p, threshold, lanes)
    }
    pub fn measure_raw(
        &mut self,
        caps: &[u16],
        freq: &[u32; 256],
        n: u32,
        p: u32,
        threshold: u32,
        lanes: usize,
    ) -> Option<[u64; 6]> {
        let t0 = crate::now();
        let (quote, m) = self.base.prepare_counted(caps, n, freq)?;
        self.mark = self.base.transfer_mark(p, quote)?;
        self.quote = quote;
        self.tokens = m;
        self.base.threshold = threshold;
        let t1 = crate::now();
        self.base.build(quote, m);
        let t2 = crate::now();
        self.base.inverse_merge();
        let t3 = crate::now();
        self.publish();
        let t4 = crate::now();
        self.index();
        let t5 = crate::now();
        self.resolve(lanes);
        std::hint::black_box((&self.stage, &self.payload, &self.gaps, &self.owners));
        let t6 = crate::now();
        Some([t1 - t0, t2 - t1, t3 - t2, t4 - t3, t5 - t4, t6 - t5])
    }

    fn publish(&mut self) {
        let c = &mut self.base;
        self.seeds.clear();
        self.hot.clear();
        self.gaps.clear();
        self.seams.clear();
        for node in &mut c.nodes {
            node.used = 0;
        }
        let pieces = &c.merged[c.nodes[0].out as usize..];
        for x in pieces {
            c.nodes[x.id as usize].used += x.width;
        }
        for (i, node) in c.nodes.iter().enumerate() {
            if node.hi - node.lo <= c.threshold {
                self.seeds.push(Range {
                    lo: node.lo,
                    hi: node.hi,
                    flat: i as u32,
                });
            } else if node.word != NONE {
                self.hot.push(Range {
                    lo: node.base,
                    hi: node.base + node.used,
                    flat: 0,
                });
            }
        }
        self.seeds.sort_unstable_by_key(|s| s.lo);
        self.hot.sort_unstable_by_key(|s| s.lo);
        let mut csum = 0;
        for seed in &mut self.seeds {
            c.nodes[seed.flat as usize].cursor = csum;
            seed.flat = csum;
            csum += seed.hi - seed.lo;
            if self.seams.last() == Some(&seed.lo) {
                self.seams.pop();
            } else {
                self.seams.push(seed.lo);
            }
            self.seams.push(seed.hi);
        }
        self.owners.resize(csum as usize, Piece::default());
        self.stage.resize(csum as usize, StageWord::default());
        let mut dst = 0;
        for x in pieces {
            let node = &mut c.nodes[x.id as usize];
            if node.word == NONE {
                for j in 0..x.width {
                    self.owners[(node.cursor + j) as usize] = Piece {
                        id: x.id,
                        width: dst + j,
                    };
                }
                node.cursor += x.width;
            }
            dst += x.width;
        }
        let mut prev = 0;
        let mut flat = 0;
        for r in &self.hot {
            if prev < r.lo {
                self.gaps.push(Range {
                    lo: prev,
                    hi: r.lo,
                    flat,
                });
                flat += r.lo - prev;
            }
            prev = r.hi;
        }
        if prev < self.tokens {
            self.gaps.push(Range {
                lo: prev,
                hi: self.tokens,
                flat,
            });
            flat += self.tokens - prev;
        }
        debug_assert_eq!(flat, csum);
    }

    fn segments(runs: &[Run], seams: &[u32], mut f: impl FnMut(Run, u32, u32, bool)) {
        let mut lo = 0;
        let mut split = 0;
        let mut inside = false;
        for &r in runs {
            let end = r.position();
            while lo < end {
                if seams.get(split) == Some(&lo) {
                    inside = !inside;
                    split += 1;
                }
                let hi = end.min(seams.get(split).copied().unwrap_or(u32::MAX));
                f(r, lo, hi, inside);
                lo = hi;
            }
        }
    }
    fn index(&mut self) {
        let c = &mut self.base;
        let mut count = [0u32; 256];
        Self::segments(&c.runs, &self.seams, |r, _, _, _| {
            count[r.byte() as usize] += 1
        });
        let mut total = 0;
        let mut slots = count.map(|n| {
            let p = total;
            total += n;
            p
        });
        self.direction.resize(total as usize, Directed::default());
        c.starts.fill(0);
        Self::segments(&c.runs, &self.seams, |r, a, b, inside| {
            let src = r.map(a);
            let slot = &mut slots[r.byte() as usize];
            let end = src + (b - a);
            self.direction[*slot as usize] = Directed {
                end_tag: (end << 9) | (u32::from(inside) << 8) | u32::from(r.byte()),
                delta: a.wrapping_sub(src),
            };
            *slot += 1;
            if src & 63 != 0 {
                c.starts[(src >> 6) as usize] |= 1u64 << (src & 63);
            }
        });
        let mut k = 0;
        for (page, first) in c.page_first.iter_mut().enumerate() {
            let row = (page as u32) * 64;
            while self.direction[k].end() <= row {
                k += 1;
            }
            *first = k as u32;
        }
    }
    fn resolve(&mut self, lanes: usize) {
        let c = &self.base;
        self.payload.clear();
        self.spans = 0;
        let mut work = [Frame::default(); 8];
        for lane in &mut self.lanes {
            lane.pending.clear();
            lane.word.clear();
        }
        let all = (1u32 << lanes) - 1;
        let mut active = 0;
        let mut next = 0;
        while active != 0 || next < self.gaps.len() {
            let mut idle = all ^ active;
            while idle != 0 && next < self.gaps.len() {
                let i = idle.trailing_zeros() as usize;
                idle &= idle - 1;
                let g = self.gaps[next];
                next += 1;
                let lo = self.quote + g.lo;
                work[i] = Frame {
                    lo,
                    hi: self.quote + g.hi,
                    slot: g.flat,
                    desc: c.locate(lo) as u32,
                    depth: 0,
                };
                active |= 1 << i;
            }
            let mut mask = active;
            while mask != 0 {
                let i = mask.trailing_zeros() as usize;
                mask &= mask - 1;
                let a = work[i];
                let cell = self.direction[a.desc as usize];
                let end = a.hi.min(cell.end());
                let width = end - a.lo;
                let dst = cell.map(a.lo);
                let lane = &mut self.lanes[i];
                if end < a.hi {
                    lane.pending.push(Frame {
                        lo: end,
                        hi: a.hi,
                        slot: a.slot + width,
                        desc: a.desc + 1,
                        depth: a.depth,
                    });
                }
                lane.word.truncate(a.depth as usize);
                lane.word.push(cell.byte());
                if cell.hit() {
                    let bytes = self.payload.len() as u32;
                    self.payload.extend_from_slice(&lane.word);
                    self.spans += 1;
                    for j in 0..width {
                        self.stage[(a.slot + j) as usize] = StageWord {
                            seed: dst + j,
                            bytes,
                            length: a.depth + 1,
                        };
                    }
                    if let Some(frame) = lane.pending.pop() {
                        work[i] = frame;
                    } else {
                        active ^= 1 << i;
                    }
                } else {
                    work[i] = Frame {
                        lo: dst,
                        hi: dst + width,
                        slot: a.slot,
                        desc: c.locate(dst) as u32,
                        depth: a.depth + 1,
                    };
                }
            }
        }
    }
    pub fn memory(&self) -> [usize; 5] {
        use crate::cap;
        let shared = self.base.stage_memory();
        let index = cap(&self.direction);
        let endpoints = cap(&self.seeds)
            + cap(&self.hot)
            + cap(&self.seams)
            + cap(&self.gaps)
            + cap(&self.owners);
        let payload = cap(&self.stage) + cap(&self.payload);
        let lanes = self
            .lanes
            .iter()
            .map(|x| cap(&x.word) + cap(&x.pending))
            .sum();
        [shared, index, endpoints, payload, lanes]
    }
    pub fn geometry(&self) -> [usize; 8] {
        [
            self.base.runs.len(),
            self.direction.len(),
            self.seeds.len(),
            self.gaps.len(),
            self.stage.len(),
            self.spans as usize,
            self.payload.len(),
            self.base.dict.len(),
        ]
    }
    pub fn verify(&self, last: &[u8], expected: &[u8]) {
        // Independent stable byte Psi and every recovered first return.
        let c = &self.base;
        let mut counts = [0usize; 256];
        for &b in last {
            counts[b as usize] += 1;
        }
        let mut at = 0;
        let mut cursor = counts.map(|n| {
            let base = at;
            at += n;
            base
        });
        let mut psi = vec![0usize; last.len()];
        for (i, &b) in last.iter().enumerate() {
            psi[cursor[b as usize]] = i;
            cursor[b as usize] += 1;
        }
        let m = self.tokens as usize;
        let mut dest = vec![usize::MAX; m];
        let mut words = vec![Vec::<u8>::new(); m];
        let mut used = vec![0usize; c.nodes.len()];
        let mut rank = 0usize;
        for piece in &c.merged[c.nodes[0].out as usize..] {
            let i = piece.id as usize;
            let node = c.nodes[i];
            if node.word != NONE {
                let w = c.words[node.word as usize];
                let d = &c.dict[w.id as usize..(w.id + w.width) as usize];
                assert_eq!(d.last(), Some(&QUOTE));
                for j in 0..piece.width as usize {
                    let s = node.base as usize + used[i] + j;
                    dest[s] = rank + j;
                    words[s].push(QUOTE);
                    words[s].extend_from_slice(&d[..d.len() - 1]);
                }
                used[i] += piece.width as usize;
            }
            rank += piece.width as usize;
        }
        assert_eq!(rank, m);
        for gap in &self.gaps {
            for ord in 0..gap.hi - gap.lo {
                let i = (gap.lo + ord) as usize;
                let frag = self.stage[(gap.flat + ord) as usize];
                let k = self.seeds.partition_point(|g| g.hi <= frag.seed);
                let seed = self.seeds[k];
                assert!(seed.lo <= frag.seed);
                let owner = self.owners[(seed.flat + frag.seed - seed.lo) as usize];
                dest[i] = owner.width as usize;
                let w = &mut words[i];
                w.extend_from_slice(
                    &self.payload[frag.bytes as usize..(frag.bytes + frag.length) as usize],
                );
                let mut v = owner.id;
                while v != NONE {
                    let node = c.nodes[v as usize];
                    w.extend(c.edges[node.edge as usize..node.end as usize].iter().rev());
                    v = node.parent;
                }
                assert_eq!(w.pop(), Some(QUOTE));
            }
        }
        let q = self.quote as usize;
        for s in 0..m {
            let mut x = q + s;
            for &byte in &words[s] {
                x = psi[x];
                assert_eq!(last[x], byte);
            }
            assert_eq!(x, q + dest[s]);
        }
        let (mut row, skip) = self.mark;
        let mut offset = (skip + 1) as usize;
        let mut out = Vec::with_capacity(last.len());
        while out.len() < last.len() {
            let w = &words[row as usize];
            let omit = offset.min(w.len());
            offset -= omit;
            let n = (last.len() - out.len()).min(w.len() - omit);
            out.extend_from_slice(&w[omit..omit + n]);
            row = dest[row as usize] as u32;
        }
        assert_eq!(out, expected);
    }
}
