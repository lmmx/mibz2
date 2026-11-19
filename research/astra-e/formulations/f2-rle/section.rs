/// Sparse occurrences of every monochrome four-letter word and their five forward steps.
#[derive(Clone, Copy, Default)]
struct Item {
    start: u32,
    rank: u32,
    len: u32,
}
pub struct Hazards {
    runs: [Vec<Item>; 256],
    base: [u32; 256],
    pub events: Vec<u32>,
    pub danger: Vec<u32>,
    pub seal: Vec<u32>,
    pub probes: usize,
    pub select: usize,
}
impl Hazards {
    pub fn new() -> Self {
        Self {
            runs: std::array::from_fn(|_| Vec::new()),
            base: [0; 256],
            events: Vec::new(),
            danger: Vec::new(),
            seal: Vec::new(),
            probes: 0,
            select: 0,
        }
    }
    pub fn build(&mut self, caps: &[u16], limit: usize) -> bool {
        for v in &mut self.runs {
            v.clear()
        }
        self.events.clear();
        self.danger.clear();
        self.seal.clear();
        self.probes = 0;
        self.select = 0;
        let mut count = [0; 256];
        let mut pos = 0u32;
        for &rec in caps {
            let c = (rec >> 8) as usize;
            let len = u32::from(rec & 255);
            let bucket = &mut self.runs[c];
            if let Some(last) = bucket
                .last_mut()
                .filter(|last| last.start + last.len == pos)
            {
                last.len += len;
            } else {
                bucket.push(Item {
                    start: pos,
                    len,
                    rank: count[c],
                });
            }
            count[c] += len;
            pos += len;
        }
        let mut prefix = 0u32;
        for c in 0..256 {
            self.base[c] = prefix;
            prefix += count[c];
            let (mut lo, mut hi) = (self.base[c], prefix);
            for _ in 1..4 {
                if lo == hi {
                    break;
                }
                let bucket = &self.runs[c];
                let rank = |x: u32| {
                    let i = bucket.partition_point(|v| v.start < x);
                    if i == 0 {
                        0
                    } else {
                        let v = bucket[i - 1];
                        v.rank + (x - v.start).min(v.len)
                    }
                };
                self.probes += 2;
                lo = self.base[c] + rank(lo);
                hi = self.base[c] + rank(hi);
            }
            if self.events.len() + (hi - lo) as usize > limit {
                return false;
            }
            self.events.extend(lo..hi);
        }
        self.danger.reserve(5 * self.events.len());
        self.seal.reserve(6 * self.events.len());
        for &x in &self.events {
            let mut y = x;
            for _ in 0..5 {
                self.danger.push(y);
                let c = self.base.partition_point(|&v| v <= y) - 1;
                let k = y - self.base[c];
                let bucket = &self.runs[c];
                let i = bucket.partition_point(|v| v.rank <= k) - 1;
                y = bucket[i].start + (k - bucket[i].rank);
                self.select += 1;
            }
            self.seal.push(y);
        }
        self.danger.sort_unstable();
        self.danger.dedup();
        self.seal.extend_from_slice(&self.danger);
        self.seal.sort_unstable();
        self.seal.dedup();
        true
    }
    pub fn index_bytes(&self) -> usize {
        self.runs
            .iter()
            .map(|v| v.capacity() * std::mem::size_of::<Item>())
            .sum::<usize>()
            + self.base.len() * 4
            + (self.events.capacity() + self.danger.capacity() + self.seal.capacity()) * 4
            + std::mem::size_of_val(&self.runs)
    }
}
