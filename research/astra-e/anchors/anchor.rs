// A high bit unused by every literal-code edge, even with 20-bit next rows.
const TAG: u32 = 1 << 31;
const LANES: usize = 4;

#[derive(Default, Copy, Clone)]
struct Anchor {
    successor: u32,
    code: u32,
    begin: u32,
}
#[derive(Default)]
struct AnchorScratch {
    entries: Vec<Anchor>,
    tails: [Vec<u8>; LANES],
}

impl AnchorScratch {
    // The input is a permutation. Mark every word and the primary before walking.
    // A tag in the old row space maps directly to its compact anchor index.
    fn build(&mut self, edges: &mut [u32], extra: &[u32], mark: u32, bits: u32) -> usize {
        self.entries.clear();
        for tail in &mut self.tails {
            tail.clear();
        }
        let mask = (1 << bits) - 1;
        let mut marked_anchor = 0;
        for (i, edge) in edges.iter_mut().enumerate() {
            let code = if extra.is_empty() { *edge >> bits } else { extra[i] };
            let next = *edge & mask;
            if code >= 256 || i as u32 == mark {
                let id = self.entries.len();
                if i as u32 == mark {
                    marked_anchor = id;
                }
                self.entries.push(Anchor { successor: next, code, begin: 0 });
                *edge = TAG | id as u32;
            } else if !extra.is_empty() {
                *edge = (code << bits) | next;
            }
        }

        // Each lane owns a fixed residue class of compact anchor IDs and a contiguous
        // literal tape. It takes its next anchor as soon as its current string ends.
        let count = self.entries.len();
        let mut current = std::array::from_fn::<_, LANES, _>(|lane| lane);
        let mut row = [0u32; LANES];
        let mut active = count.min(LANES);
        for lane in 0..active {
            row[lane] = self.entries[lane].successor;
        }
        while active != 0 {
            for lane in 0..LANES {
                let id = current[lane];
                if id >= count {
                    continue;
                }
                let edge = edges[row[lane] as usize];
                if edge & TAG == 0 {
                    self.tails[lane].push((edge >> bits) as u8);
                    row[lane] = edge & mask;
                } else {
                    self.entries[id].successor = edge & !TAG;
                    let next = id + LANES;
                    current[lane] = next;
                    if next < count {
                        self.entries[next].begin = self.tails[lane].len() as u32;
                        row[lane] = self.entries[next].successor;
                    } else {
                        active -= 1;
                    }
                }
            }
        }
        // One past-the-end offset in every tape. The end of any anchor's tail
        // is the begin offset of the anchor exactly LANES entries after it.
        for i in count..count + LANES {
            self.entries.push(Anchor { begin: self.tails[i % LANES].len() as u32, ..Anchor::default() });
        }
        marked_anchor
    }

    fn emit<F: FnMut(u32, &[u8])>(&self, mark: usize, mut visit: F) {
        let mut id = mark;
        loop {
            let anchor = self.entries[id];
            let end = self.entries[id + LANES].begin;
            visit(anchor.code, &self.tails[id % LANES][anchor.begin as usize..end as usize]);
            id = anchor.successor as usize;
            if id == mark {
                break;
            }
        }
    }
}
