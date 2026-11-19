use std::mem::MaybeUninit;

pub struct Table {
    pub edges: Vec<u32>,
    pub extra: Vec<u32>,
    pub spelling: Vec<u8>,
    pub offsets: Vec<u32>,
    pub n: usize,
    pub mark: u32,
    pub bits: u32,
}
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Job {
    pub row: u32,
    pub start: u32,
}
#[derive(Copy, Clone, Default)]
struct Link {
    next: u32,
    length: u32,
}
#[derive(Default)]
pub struct Plan {
    links: Vec<Link>,
    pub jobs: Vec<Job>,
    pub period: u32,
}

impl Table {
    #[inline(always)]
    fn width(&self, code: u32) -> u32 {
        if code < 256 {
            return 1;
        }
        let id = (code - 256) as usize;
        self.offsets[id + 1] - self.offsets[id]
    }
    #[inline(always)]
    fn code<const WIDE: bool>(&self, row: u32, e: u32) -> u32 {
        if WIDE {
            self.extra[row as usize]
        } else {
            e >> self.bits
        }
    }
    #[inline(always)]
    fn store(&self, code: u32, pos: u32, out: &mut [MaybeUninit<u8>]) -> u32 {
        if code < 256 {
            out[pos as usize].write(code as u8);
            pos + 1
        } else {
            let id = (code - 256) as usize;
            let text = &self.spelling[self.offsets[id] as usize..self.offsets[id + 1] as usize];
            let end = pos + text.len() as u32;
            let dst = &mut out[pos as usize..end as usize];
            // The checked source and destination live in distinct allocations.
            unsafe {
                std::ptr::copy_nonoverlapping(text.as_ptr(), dst.as_mut_ptr().cast(), text.len());
            }
            end
        }
    }

    // Independent serial oracle: recover starts directly on the original marked cycle.
    pub fn oracle(&self, shift: u32) -> Plan {
        let stride = (1u32 << shift) - 1;
        let low = self.mark & stride;
        let mask = (1 << self.bits) - 1;
        let mut row = self.mark;
        let mut result = Plan::default();
        loop {
            if row & stride == low {
                result.jobs.push(Job {
                    row,
                    start: result.period,
                });
            }
            let e = self.edges[row as usize];
            let code = if self.extra.is_empty() {
                e >> self.bits
            } else {
                self.extra[row as usize]
            };
            result.period += self.width(code);
            row = e & mask;
            if row == self.mark {
                break;
            }
        }
        result
    }
}

impl Plan {
    pub fn capacity_bytes(&self) -> usize {
        self.links.capacity() * 8 + self.jobs.capacity() * 8
    }
    pub fn landmarks(&self) -> usize {
        self.links.len()
    }
    pub fn build<const LANES: usize, const SHIFT: u32, const WIDE: bool>(&mut self, t: &Table) {
        let stride = (1u32 << SHIFT) - 1;
        let low = t.mark & stride;
        let mask = (1 << t.bits) - 1;
        let n = ((t.edges.len() as u32 - 1 - low) >> SHIFT) + 1;
        self.links.resize(n as usize, Link::default());
        let mut current = std::array::from_fn::<u32, LANES, _>(|i| i as u32);
        let mut row = current.map(|i| low + (i << SHIFT));
        let mut lengths = [0u32; LANES];
        let mut next = LANES as u32;
        let mut done = 0;
        while done < n {
            for lane in 0..LANES {
                let id = current[lane];
                if id >= n {
                    continue;
                }
                let r = row[lane];
                let e = t.edges[r as usize];
                lengths[lane] += t.width(t.code::<WIDE>(r, e));
                let r = e & mask;
                row[lane] = r;
                if r & stride == low {
                    self.links[id as usize] = Link {
                        next: r >> SHIFT,
                        length: lengths[lane],
                    };
                    done += 1;
                    current[lane] = next;
                    row[lane] = low + (next << SHIFT);
                    lengths[lane] = 0;
                    next += 1;
                }
            }
        }

        self.jobs.clear();
        let first = t.mark >> SHIFT;
        let mut id = first;
        let mut pos = 0;
        loop {
            self.jobs.push(Job {
                row: low + (id << SHIFT),
                start: pos,
            });
            let l = self.links[id as usize];
            pos += l.length;
            id = l.next;
            if id == first {
                break;
            }
        }
        self.period = pos;
    }

    pub fn write<const LANES: usize, const WIDE: bool>(&self, t: &Table, out: &mut Vec<u8>) {
        let p = self.period as usize;
        assert!(p > 0 && p <= t.n);
        out.clear();
        out.reserve(t.n);
        let dst = &mut out.spare_capacity_mut()[..p];
        let mask = (1 << t.bits) - 1;
        let mut row = [0u32; LANES];
        let mut pos = [0u32; LANES];
        let mut stop = [0u32; LANES];
        let mut assigned = 0;
        let mut completed = 0;
        for lane in 0..LANES.min(self.jobs.len()) {
            let j = self.jobs[lane];
            row[lane] = j.row;
            pos[lane] = j.start;
            stop[lane] = self.jobs.get(lane + 1).map_or(self.period, |v| v.start);
            assigned += 1;
        }
        while completed < self.jobs.len() {
            for lane in 0..LANES {
                if pos[lane] == stop[lane] {
                    continue;
                }
                let r = row[lane];
                let e = t.edges[r as usize];
                row[lane] = e & mask;
                pos[lane] = t.store(t.code::<WIDE>(r, e), pos[lane], dst);
                if pos[lane] == stop[lane] {
                    completed += 1;
                    if let Some(j) = self.jobs.get(assigned) {
                        row[lane] = j.row;
                        pos[lane] = j.start;
                        stop[lane] = self.jobs.get(assigned + 1).map_or(self.period, |v| v.start);
                        assigned += 1;
                    }
                }
            }
        }
        // Every primary segment was completed. Its immutable label sequence writes
        // exactly [start_i,start_{i+1}), partitioning the initialized prefix.
        unsafe {
            out.set_len(p);
        }
        while out.len() < t.n {
            let sz = out.len().min(t.n - out.len());
            out.extend_from_within(..sz);
        }
    }
}

pub fn paid<const LANES: usize, const SHIFT: u32>(plan: &mut Plan, t: &Table, out: &mut Vec<u8>) {
    if t.extra.is_empty() {
        plan.build::<LANES, SHIFT, false>(t);
        plan.write::<LANES, false>(t, out);
    } else {
        plan.build::<LANES, SHIFT, true>(t);
        plan.write::<LANES, true>(t, out);
    }
}
pub fn oracle_write<const LANES: usize>(plan: &Plan, t: &Table, out: &mut Vec<u8>) {
    if t.extra.is_empty() {
        plan.write::<LANES, false>(t, out);
    } else {
        plan.write::<LANES, true>(t, out);
    }
}
