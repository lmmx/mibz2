use mibz2::{
    runs,
    stream::{Event, Scanner, SliceSource},
    Kernels,
};
use std::{
    env, fs,
    io::{BufWriter, Write},
    path::Path,
};

include!("../edge-budget/gate.rs");

const STREAMS: usize = 272;
const CAP: [usize; 6] = [16, 32, 64, 128, 256, 512];

fn pieces(records: &[u16], mark: u32, rows: usize) -> Vec<(usize, u32)> {
    let (mut i, mut row) = (0, 0);
    let mut out = Vec::new();
    while i < records.len() {
        let c = records[i] >> 8;
        let mut w = 0;
        while i < records.len() && records[i] >> 8 == c {
            w += u32::from(records[i] & 255);
            i += 1;
        }
        if w <= 16 {
            out.push((usize::from(c), w));
        } else if (row..row + w).contains(&mark) {
            out.push((usize::from(c), 1));
        }
        row += w;
    }
    assert_eq!(row as usize, rows);
    out
}

struct Tape {
    rows: Vec<(usize, u32)>,
    groups: [Vec<u32>; 16],
    lengths: [usize; STREAMS],
    widths: [usize; 17],
    byte_pieces: [usize; 256],
}

impl Tape {
    fn new(rows: Vec<(usize, u32)>) -> Self {
        let mut groups: [Vec<u32>; 16] = Default::default();
        let mut lengths = [0; STREAMS];
        let mut widths = [0; 17];
        let mut byte_pieces = [0; 256];
        let mut reference: [Vec<u32>; 256] = std::array::from_fn(|_| Vec::new());
        let mut rank = 0;
        for &(c, w) in &rows {
            assert!(rank < 1 << 20 && (1..=16).contains(&w));
            groups[c >> 4].push(rank | ((w - 1) << 20) | (((c & 15) as u32) << 24));
            reference[c].extend(rank..rank + w);
            lengths[c] += w as usize;
            lengths[256 + (c >> 4)] += 1;
            widths[w as usize] += 1;
            byte_pieces[c] += 1;
            rank += w;
        }
        let mut expanded: [Vec<u32>; 256] = std::array::from_fn(|_| Vec::new());
        for (high, group) in groups.iter().enumerate() {
            for &d in group {
                let (c, w, m) = decode(high, d);
                expanded[c].extend(m..m + w);
            }
        }
        assert!(reference == expanded);
        Self {
            rows,
            groups,
            lengths,
            widths,
            byte_pieces,
        }
    }
}

fn decode(high: usize, d: u32) -> (usize, u32, u32) {
    (
        (high << 4) | ((d >> 24) as usize),
        ((d >> 20) & 15) + 1,
        d & ((1 << 20) - 1),
    )
}

// One marked timestamp per address, allowing all fully associative LRU sizes
// to share a single stack-distance calculation.
struct Fenwick(Vec<i32>);
impl Fenwick {
    fn add(&mut self, mut i: usize, d: i32) {
        while i < self.0.len() {
            self.0[i] += d;
            i += i & i.wrapping_neg();
        }
    }
    fn sum(&self, mut i: usize) -> usize {
        let mut sum = 0;
        while i != 0 {
            sum += self.0[i];
            i &= i - 1;
        }
        sum as usize
    }
}

struct Model {
    offsets: [usize; STREAMS],
    sets: [usize; STREAMS],
    ids: [usize; STREAMS],
    last: Vec<usize>,
    recent: [[usize; 8]; 64],
    fenwick: Fenwick,
    time: usize,
    unique: usize,
    miss: [usize; CAP.len()],
    set_miss: [usize; 2],
}
impl Model {
    fn new(tape: &Tape, layout: usize) -> Self {
        let mut offsets = [0; STREAMS];
        let mut sets = [0; STREAMS];
        let mut ids = [0; STREAMS];
        let mut line_count = 0;
        for c in 0..STREAMS {
            let mut h = (c as u32 + 1).wrapping_mul(0x9e3779b9);
            h ^= h >> 16;
            h = h.wrapping_mul(0x85ebca6b);
            // Layout 0 = 64B-aligned, distinct base sets in byte order.
            // Layout 1 = all bases on a 4KiB boundary, stressing conflicts.
            // Layout 2 = distributed base sets with 16-byte alignment.
            (offsets[c], sets[c]) = match layout {
                0 => (0, c & 63),
                1 => (0, 0),
                2 => ((h as usize & 3) * 4, (h as usize >> 2) & 63),
                _ => unreachable!(),
            };
            ids[c] = line_count;
            line_count += (offsets[c] + tape.lengths[c]).div_ceil(16);
        }
        Self {
            offsets,
            sets,
            ids,
            last: vec![0; line_count],
            recent: [[usize::MAX; 8]; 64],
            fenwick: Fenwick(vec![0; tape.rows.len() * 4 + 2]),
            time: 0,
            unique: 0,
            miss: [0; CAP.len()],
            set_miss: [0; 2],
        }
    }
    fn touch(&mut self, c: usize, word: usize, width: usize) {
        let first = (self.offsets[c] + word) / 16;
        let last = (self.offsets[c] + word + width - 1) / 16;
        for line in first..=last {
            self.time += 1;
            let id = self.ids[c] + line;
            let prior = self.last[id];
            let distance = if prior == 0 {
                self.unique += 1;
                usize::MAX
            } else {
                let distance = self.unique - self.fenwick.sum(prior);
                self.fenwick.add(prior, -1);
                distance
            };
            self.fenwick.add(self.time, 1);
            self.last[id] = self.time;
            for (miss, cap) in self.miss.iter_mut().zip(CAP) {
                *miss += usize::from(distance >= cap);
            }
            let rec = &mut self.recent[(self.sets[c] + line) & 63];
            let pos = rec.iter().position(|&v| v == id).unwrap_or(8);
            self.set_miss[0] += usize::from(pos >= 4);
            self.set_miss[1] += usize::from(pos >= 8);
            if pos > 0 {
                rec.copy_within(0..pos.min(7), 1);
                rec[0] = id;
            }
        }
    }
    fn replay(tape: &Tape, layout: usize, mode: &str) -> Self {
        let mut model = Self::new(tape, layout);
        let mut len = [0; STREAMS];
        if mode == "direct" {
            for &(c, w) in &tape.rows {
                model.touch(c, len[c], w as usize);
                len[c] += w as usize;
            }
        } else {
            if mode == "staged" {
                for &(c, _) in &tape.rows {
                    let h = 256 + (c >> 4);
                    model.touch(h, len[h], 1);
                    len[h] += 1;
                }
            }
            for (h, group) in tape.groups.iter().enumerate() {
                for (i, &d) in group.iter().enumerate() {
                    if mode == "staged" {
                        model.touch(256 + h, i, 1);
                    }
                    let (c, w, _) = decode(h, d);
                    model.touch(c, len[c], w as usize);
                    len[c] += w as usize;
                }
            }
        }
        assert_eq!(len[..256], tape.lengths[..256]);
        assert!(model.time < model.fenwick.0.len());
        model
    }
}

fn output(dir: &Path, name: &str) -> BufWriter<fs::File> {
    BufWriter::new(fs::File::create(dir.join(name)).unwrap())
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let dir = Path::new(&args[2]);
    fs::create_dir_all(dir).unwrap();
    let (mut geometry, mut model, mut alphabet) = (
        output(dir, "blocks.csv"),
        output(dir, "locality.csv"),
        output(dir, "alphabet.csv"),
    );
    writeln!(geometry,"ordinal,n,caps,pieces,queued_rows,singletons,short2,short3,short4,active_bytes,active_nibbles").unwrap();
    writeln!(alphabet, "ordinal,byte,pieces,rows").unwrap();
    writeln!(model,"ordinal,layout,mode,line_references,unique_lines,fa16,fa32,fa64,fa128,fa256,fa512,set64w4,set64w8").unwrap();
    let data = fs::read(&args[1]).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut records = runs::Runs::default();
    let mut totals = [0; 3];
    while let Event::Block { index, symbols, .. } = scanner.next(Vec::new()).unwrap() {
        totals[0] += 1;
        if !runs::takes_runs(&symbols) {
            continue;
        }
        totals[1] += 1;
        let n = runs::front(&symbols, &mut records, Kernels::detect()).unwrap();
        let caps = records.records();
        if caps.len() * 20 >= n * 3 && (n < 65_536 || !fits(caps, symbols.orig_ptr)) {
            continue;
        }
        totals[2] += 1;
        let t = Tape::new(pieces(caps, symbols.orig_ptr, n));
        writeln!(
            geometry,
            "{index},{n},{},{},{},{},{},{},{},{},{}",
            caps.len(),
            t.rows.len(),
            t.lengths[..256].iter().sum::<usize>(),
            t.widths[1],
            t.widths[2],
            t.widths[3],
            t.widths[4],
            t.byte_pieces.iter().filter(|&&x| x > 0).count(),
            t.groups.iter().filter(|g| !g.is_empty()).count()
        )
        .unwrap();
        for (c, &r) in t.byte_pieces.iter().enumerate().filter(|(_, r)| **r > 0) {
            writeln!(alphabet, "{index},{c},{r},{}", t.lengths[c]).unwrap();
        }
        for layout in 0..3 {
            for mode in ["direct", "grouped", "staged"] {
                let m = Model::replay(&t, layout, mode);
                write!(model, "{index},{layout},{mode},{},{}", m.time, m.unique).unwrap();
                for v in m.miss.into_iter().chain(m.set_miss) {
                    write!(model, ",{v}").unwrap();
                }
                writeln!(model).unwrap();
            }
        }
    }
    eprintln!(
        "scanned={} takes_runs={} admitted={}",
        totals[0], totals[1], totals[2]
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn retained_mark_splits_the_maximal_run_after_joining_records() {
        let runs = [0x0aff, 0x0a07, 0x0b01, 0x0c10, 0x0d11];
        for p in [0, 8, 261] {
            assert_eq!(pieces(&runs, p, 296), [(10, 1), (11, 1), (12, 16)]);
        }
        assert_eq!(pieces(&runs, 262, 296), [(11, 1), (12, 16)]);
        assert_eq!(pieces(&runs, 295, 296), [(11, 1), (12, 16), (13, 1)]);
    }

    fn reference(trace: &[(usize, usize)], sets: usize, ways: usize) -> usize {
        let mut cache = vec![VecDeque::new(); sets];
        let mut misses = 0;
        for &(id, set) in trace {
            let queue = &mut cache[set % sets];
            if let Some(p) = queue.iter().position(|&a| a == id) {
                queue.remove(p);
            } else {
                misses += 1;
                if queue.len() == ways {
                    queue.pop_back();
                }
            }
            queue.push_front(id);
        }
        misses
    }

    #[test]
    fn stack_and_set_counters_agree_with_individual_cache_simulations() {
        let mut random = 17u32;
        let mut pieces = Vec::new();
        for _ in 0..1600 {
            random = random.wrapping_mul(1664525).wrapping_add(1013904223);
            pieces.push(((random >> 24) as usize, 1 + ((random >> 20) & 15)));
        }
        let tape = Tape::new(pieces);
        for layout in 0..3 {
            for mode in ["direct", "grouped", "staged"] {
                let m = Model::replay(&tape, layout, mode);
                let mut trace = Vec::new();
                let mut emit = |c: usize, i: usize, w: usize| {
                    let mut previous = None;
                    for word in i..i + w {
                        let line = (word + m.offsets[c]) / 16;
                        if previous != Some(line) {
                            trace.push((m.ids[c] + line, m.sets[c] + line));
                            previous = Some(line);
                        }
                    }
                };
                let mut offsets = [0; STREAMS];
                if mode != "grouped" {
                    for &(c, w) in &tape.rows {
                        let (s, count) = if mode == "direct" {
                            (c, w as usize)
                        } else {
                            (256 + c / 16, 1)
                        };
                        emit(s, offsets[s], count);
                        offsets[s] += count;
                    }
                }
                if mode != "direct" {
                    for hi in 0..16 {
                        for (i, &(c, w)) in
                            tape.rows.iter().filter(|(c, _)| c / 16 == hi).enumerate()
                        {
                            if mode == "staged" {
                                emit(256 + hi, i, 1);
                            }
                            emit(c, offsets[c], w as usize);
                            offsets[c] += w as usize;
                        }
                    }
                }
                assert_eq!(m.time, trace.len());
                for (limit, count) in CAP.into_iter().zip(m.miss) {
                    assert_eq!(count, reference(&trace, 1, limit));
                }
                for (ways, count) in [4, 8].into_iter().zip(m.set_miss) {
                    assert_eq!(count, reference(&trace, 64, ways));
                }
            }
        }
    }
}
