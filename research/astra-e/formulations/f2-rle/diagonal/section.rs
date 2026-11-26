// Store only L-run intersections with the matching F bucket.
#[derive(Clone, Copy)]
struct Item {
    start: u32,
    rank: u32,
    width: u32,
}
pub struct Hazards {
    diagonal: [Vec<Item>; 256],
    base: [u32; 256],
    at_bucket: [u32; 256],
    third: Vec<u32>,
    fourth: Vec<u32>,
    fifth: Vec<u32>,
    pub events: Vec<u32>,
    pub danger: Vec<u32>,
    pub seal: Vec<u32>,
    pub probes: usize,
    pub select: usize,
}
impl Hazards {
    pub fn new() -> Self {
        Self {
            diagonal: std::array::from_fn(|_| Vec::new()),
            base: [0; 256],
            at_bucket: [0; 256],
            third: Vec::new(),
            fourth: Vec::new(),
            fifth: Vec::new(),
            events: Vec::new(),
            danger: Vec::new(),
            seal: Vec::new(),
            probes: 0,
            select: 0,
        }
    }
    pub fn index_bytes(&self) -> usize {
        self.diagonal
            .iter()
            .map(|v| v.capacity() * std::mem::size_of::<Item>())
            .sum::<usize>()
            + std::mem::size_of::<Self>()
            + [
                &self.events,
                &self.danger,
                &self.seal,
                &self.third,
                &self.fourth,
                &self.fifth,
            ]
            .into_iter()
            .map(|v| 4 * v.capacity())
            .sum::<usize>()
    }
    pub fn build(&mut self, caps: &[u16], limit: usize) -> bool {
        for v in &mut self.diagonal {
            v.clear();
        }
        self.events.clear();
        self.danger.clear();
        self.seal.clear();
        self.third.clear();
        self.probes = 0;
        self.select = 0;
        let mut counts = [0u32; 256];
        for &v in caps {
            counts[(v >> 8) as usize] += u32::from(v & 255);
        }
        let mut size = 0;
        for (base, &width) in self.base.iter_mut().zip(&counts) {
            *base = size;
            size += width;
        }
        let mut seen = [0u32; 256];
        self.at_bucket.fill(0);
        let mut f = 0usize;
        let mut pos = 0;
        for &v in caps {
            let c = (v >> 8) as usize;
            let end = pos + u32::from(v & 255);
            while pos < end {
                while f < 256 && self.base[f] + counts[f] <= pos {
                    f += 1;
                    if f < 256 {
                        self.at_bucket[f] = seen[f];
                    }
                }
                let stop = end.min(self.base[f] + counts[f]);
                if f == c {
                    let bucket = &mut self.diagonal[c];
                    if let Some(last) = bucket.last_mut().filter(|t| t.start + t.width == pos) {
                        last.width += stop - pos;
                    } else {
                        bucket.push(Item {
                            start: pos,
                            rank: seen[c],
                            width: stop - pos,
                        });
                    }
                }
                seen[c] += stop - pos;
                pos = stop;
            }
        }
        for c in 0..256 {
            let a = &self.diagonal[c];
            if a.is_empty() {
                continue;
            }
            let mut lo = self.base[c];
            let mut hi = lo + counts[c];
            let rank = |x: u32| {
                let j = a.partition_point(|t| t.start < x);
                if j == 0 {
                    self.at_bucket[c]
                } else {
                    let t = a[j - 1];
                    t.rank + (x - t.start).min(t.width)
                }
            };
            for _ in 1..4 {
                if lo == hi {
                    break;
                }
                lo = self.base[c] + rank(lo);
                hi = self.base[c] + rank(hi);
                self.probes += 2;
            }
            if self.events.len() + (hi - lo) as usize > limit {
                return false;
            }
            for x in lo..hi {
                self.events.push(x);
                let mut y = x;
                for _ in 0..3 {
                    self.danger.push(y);
                    let k = y - self.base[c];
                    let j = a.partition_point(|v| v.rank <= k) - 1;
                    let t = a[j];
                    debug_assert!(k < t.rank + t.width);
                    y = t.start + (k - t.rank);
                }
                self.third.push(y);
                self.danger.push(y);
            }
        }
        if self.events.is_empty() {
            return true;
        }
        self.select = 5 * self.events.len();
        step(caps, &self.base, size, &self.third, &mut self.fourth);
        step(caps, &self.base, size, &self.fourth, &mut self.fifth);
        self.danger.extend_from_slice(&self.fourth);
        self.danger.sort_unstable();
        self.danger.dedup();
        self.seal.extend_from_slice(&self.danger);
        self.seal.extend_from_slice(&self.fifth);
        self.seal.sort_unstable();
        self.seal.dedup();
        true
    }
}
fn step(caps: &[u16], base: &[u32; 256], size: u32, from: &[u32], into: &mut Vec<u32>) {
    into.clear();
    let mut next = [0usize; 256];
    let mut cap = [0usize; 256];
    let mut at = 0;
    for c in 0..256 {
        next[c] = at;
        let end = base.get(c + 1).copied().unwrap_or(size);
        while at < from.len() && from[at] < end {
            at += 1;
        }
        cap[c] = at;
    }
    let mut rank = *base;
    let mut pos = 0;
    for &v in caps {
        let c = (v >> 8) as usize;
        let w = u32::from(v & 255);
        let end = rank[c] + w;
        while next[c] < cap[c] && from[next[c]] < end {
            into.push(pos + (from[next[c]] - rank[c]));
            next[c] += 1;
        }
        pos += w;
        rank[c] = end;
        if into.len() == from.len() {
            break;
        }
    }
    assert_eq!(into.len(), from.len());
}
