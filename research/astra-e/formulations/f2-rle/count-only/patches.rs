#[derive(Clone, Copy)]
struct Edit {
    rank: u32,
    offset: u32,
    byte: u8,
    count: u8,
}
impl Kernel {
    fn kept(&self, row: u32) -> bool {
        let at = self.strips.partition_point(|s| s.from <= row);
        at == 0 || row - self.strips[at - 1].from >= self.strips[at - 1].width
    }
    fn source_code(&self, rank: usize, bits: u32) -> u32 {
        if self.extra.is_empty() {
            self.edges[rank] >> bits
        } else {
            self.extra[rank]
        }
    }
    fn set_code(&mut self, rank: usize, bits: u32, code: u32) {
        if self.extra.is_empty() {
            self.edges[rank] = (self.edges[rank] & ((1 << bits) - 1)) | (code << bits);
        } else {
            self.extra[rank] = code;
        }
    }
    fn code_len(&self, code: u32) -> u32 {
        if code < 256 {
            1
        } else {
            let i = (code - 256) as usize;
            self.offsets[i + 1] - self.offsets[i]
        }
    }
    fn extend_raw_part(&mut self, code: u32, start: u32, end: u32) {
        if start == end {
            return;
        }
        if code < 256 {
            debug_assert_eq!((start, end), (0, 1));
            self.spelling.push(code as u8);
        } else {
            let off = self.offsets[(code - 256) as usize];
            self.spelling
                .extend_from_within((off + start) as usize..(off + end) as usize);
        }
    }
    fn edits(&mut self, n: u32, p: u32) -> Option<i64> {
        self.steps = 0;
        self.edits.clear();
        self.danger.clear();
        self.added = 0;
        let mut delta = 0i64;
        let mut begin = 0;
        while begin < self.events.len() {
            let mut a = self.events[begin];
            let mut end = a + 1;
            begin += 1;
            while begin < self.events.len() && self.events[begin] == end {
                begin += 1;
                end += 1;
            }
            while a < end {
                let mut x = a;
                let mut width = end - a;
                let mut depth = 0;
                let owner = loop {
                    let j = self.strips.partition_point(|s| s.from <= x);
                    if j == 0 || x - self.strips[j - 1].from >= self.strips[j - 1].width {
                        width = width.min(self.strips.get(j).map_or(n, |s| s.from) - x);
                        break Some(x);
                    }
                    let s = &self.strips[j - 1];
                    width = width.min(s.from + s.width - x);
                    x = s.to + (x - s.from);
                    depth += 1;
                    self.steps += 1;
                    if self.steps > 4096 {
                        self.reason = 2;
                        return None;
                    }
                    if x == a {
                        break None;
                    }
                };
                for off in 0..width {
                    let mut y = a + off;
                    let b = self.fbase.partition_point(|&v| v <= y) - 1;
                    let mut source = owner.map(|first| first + off);
                    let mut offset = depth;
                    for step in 0..5 {
                        self.danger.push(y);
                        if step == 4 {
                            break;
                        }
                        y = self.psi(y);
                        offset += 1;
                        if self.kept(y) {
                            source = Some(y);
                            offset = 0;
                        }
                    }
                    let count = (self.fbase.partition_point(|&v| v <= y) - 1) as u8;
                    delta += i64::from(count) - 1;
                    if let Some(row) = source {
                        self.edits.push(Edit {
                            rank: row - self.prefix_s(row),
                            offset,
                            byte: b as u8,
                            count,
                        });
                    }
                }
                a += width;
            }
        }
        self.danger.sort_unstable();
        self.danger.dedup();
        if self.danger.len() != 5 * self.events.len() {
            self.reason = 4;
            return None;
        }
        if self.danger.binary_search(&p).is_ok() {
            self.reason = 5;
            return None;
        }
        Some(delta)
    }
    fn patch(&mut self, bits: u32, budget: u32) -> bool {
        self.owners.clear();
        self.labels.clear();
        self.edits.sort_unstable_by_key(|v| (v.rank, v.offset));
        let mut i = 0;
        let mut total = 0i64;
        while i < self.edits.len() {
            let row = self.edits[i].rank;
            let code = self.source_code(row as usize, bits);
            total += i64::from(self.code_len(code));
            self.owners.push(row);
            self.labels.push(code);
            while i < self.edits.len() && self.edits[i].rank == row {
                let e = self.edits[i];
                total += i64::from(e.count) - 1;
                i += 1;
            }
        }
        self.added = total as usize;
        if total > i64::from(budget) {
            self.reason = 6;
            return false;
        }
        let first = 255 + self.offsets.len() as u32;
        if self.extra.is_empty()
            && !self.owners.is_empty()
            && first + self.owners.len() as u32 - 1 > u32::MAX >> bits
        {
            self.reason = 3;
            return false;
        }
        i = 0;
        for oi in 0..self.owners.len() {
            let row = self.owners[oi];
            let code = self.labels[oi];
            let mut from = 0;
            while i < self.edits.len() && self.edits[i].rank == row {
                let e = self.edits[i];
                self.extend_raw_part(code, from, e.offset);
                self.spelling
                    .resize(self.spelling.len() + usize::from(e.count), e.byte);
                from = e.offset + 1;
                i += 1;
            }
            self.extend_raw_part(code, from, self.code_len(code));
            self.offsets.push(self.spelling.len() as u32);
            self.set_code(row as usize, bits, first + oi as u32);
        }
        true
    }
    fn write_once(&self, mark: u32, bits: u32, out: &mut Vec<u8>) {
        let mask = (1 << bits) - 1;
        let mut row = mark;
        let mut edge = self.edges[mark as usize];
        loop {
            let code = if self.extra.is_empty() {
                edge >> bits
            } else {
                self.extra[row as usize]
            };
            row = edge & mask;
            edge = self.edges[row as usize];
            if code < 256 {
                out.push(code as u8);
            } else {
                let w = (code - 256) as usize;
                out.extend_from_slice(
                    &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize],
                );
            }
            if row == mark {
                break;
            }
        }
    }
    pub fn expanded(
        &mut self,
        caps: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: super::Kernels,
    ) -> Result<(), mibz2::ErrorKind> {
        let (_, mark, bits) = self.construct::<true, true>(caps, n as u32, p, 16);
        self.reason = 0;
        self.steps = 0;
        self.added = 0;
        self.labels.clear();
        if self.event_ranges(n as u32) {
            if let Some(delta) = self.edits(n as u32, p) {
                if self.patch(bits, n as u32 / 16) {
                    let start = out.len();
                    let total = (n as i64 + delta) as usize;
                    out.reserve(total);
                    self.write_once(mark, bits, out);
                    if out.len() - start == total {
                        return Ok(());
                    }
                    self.reason = 7;
                    out.truncate(start);
                    for i in 0..self.owners.len() {
                        self.set_code(self.owners[i] as usize, bits, self.labels[i]);
                    }
                }
            }
        } else {
            self.reason = 1;
        }
        emit(
            &self.edges,
            &self.extra,
            &self.spelling,
            &self.offsets,
            n,
            mark,
            bits,
            &mut self.raw,
        );
        mibz2::rle::unrle(&self.raw, out, k)
    }
    pub fn observations(&self) -> (u8, usize, usize, usize, usize) {
        (
            self.reason,
            self.events.len(),
            self.steps,
            self.labels.len(),
            self.added,
        )
    }
}
