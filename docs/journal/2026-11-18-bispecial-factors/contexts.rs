//! A reference encoding of a cyclic byte BWT by weighted two-sided extension graphs.
//! Compile with rustc -O --edition 2021 contexts.rs. See the companion README.

#[path = "support.rs"]
mod support;

use std::{collections::VecDeque, env, fs, io::Write};

const ALPHABET: usize = 256;
const MAX_BLOCK: usize = 900_000;
const MAGIC: &[u8; 4] = b"EXT1";

#[derive(Clone, Copy)]
struct Band {
    byte: u8,
    count: usize,
    // First row in the stable LF image of these occurrences.
    image: usize,
}

#[derive(Default)]
enum Cell {
    #[default]
    Empty,
    One(Band),
    Mixed(Vec<Band>),
}

impl Cell {
    fn new(mut bands: Vec<Band>) -> Self {
        match bands.len() {
            0 => panic!("empty incoming histogram"),
            1 => Self::One(bands.pop().unwrap()),
            _ => Self::Mixed(bands),
        }
    }

    fn bands(&self) -> &[Band] {
        match self {
            Self::Empty => &[],
            Self::One(b) => std::slice::from_ref(b),
            Self::Mixed(bs) => bs,
        }
    }
}

// Right extensions, in lexicographic order. Names of factors never enter the tape.
struct Split {
    start: usize,
    widths: Vec<usize>,
}

#[derive(Default)]
struct Tape {
    support: Vec<u8>,
    bits: usize,
    weights: Vec<u8>,
}

impl Tape {
    fn write_bit(&mut self, value: bool) {
        if self.bits % 8 == 0 {
            self.support.push(0);
        }
        if value {
            self.support[self.bits / 8] |= 1 << (self.bits % 8);
        }
        self.bits += 1;
    }

    fn write_weight(&mut self, mut value: usize) {
        while value >= 128 {
            self.weights.push((value as u8 & 127) | 128);
            value >>= 7;
        }
        self.weights.push(value as u8);
    }

    fn serialize(&self, counts: &[usize; ALPHABET]) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        for &n in counts {
            out.extend((n as u32).to_le_bytes());
        }
        out.extend((self.bits as u64).to_le_bytes());
        out.extend(&self.support);
        out.extend(&self.weights);
        out
    }

    fn parse(input: &[u8]) -> ([usize; ALPHABET], Self) {
        let header = 4 + 4 * ALPHABET + 8;
        assert!(input.len() >= header && &input[..4] == MAGIC);
        let mut counts = [0; ALPHABET];
        for (n, bytes) in counts.iter_mut().zip(input[4..].chunks_exact(4)) {
            *n = u32::from_le_bytes(bytes.try_into().unwrap()) as usize;
        }
        let bits = u64::from_le_bytes(input[header - 8..header].try_into().unwrap()) as usize;
        let (support, weights) = input[header..].split_at(bits.div_ceil(8));
        (
            counts,
            Self {
                support: support.to_vec(),
                bits,
                weights: weights.to_vec(),
            },
        )
    }
}

enum Mode<'a> {
    Pack {
        positions: Vec<Vec<usize>>,
        tape: &'a mut Tape,
    },
    Unpack {
        tape: &'a Tape,
        bit: usize,
        weight: usize,
    },
}

impl Mode<'_> {
    fn present(&mut self, known: usize) -> bool {
        match self {
            Self::Pack { tape, .. } => {
                tape.write_bit(known > 0);
                known > 0
            }
            Self::Unpack { tape, bit, .. } => {
                assert!(*bit < tape.bits);
                let yes = tape.support[*bit / 8] & (1 << (*bit % 8)) != 0;
                *bit += 1;
                yes
            }
        }
    }

    fn residual(&mut self, known: usize, limit: usize) -> usize {
        if limit == 0 {
            return 0;
        }
        let value = match self {
            Self::Pack { tape, .. } => {
                tape.write_weight(known);
                known
            }
            Self::Unpack { tape, weight, .. } => {
                let mut x = 0;
                for shift in [0, 7, 14] {
                    let b = tape.weights[*weight];
                    *weight += 1;
                    x |= ((b & 127) as usize) << shift;
                    if b < 128 {
                        assert!(x <= limit);
                        return x;
                    }
                }
                panic!("weight exceeds the bzip2 block bound")
            }
        };
        assert!(value <= limit);
        value
    }
}

struct Forest(Vec<usize>);

impl Forest {
    fn root(&mut self, mut u: usize) -> usize {
        while self.0[u] != u {
            self.0[u] = self.0[self.0[u]];
            u = self.0[u];
        }
        u
    }

    fn join(&mut self, u: usize, v: usize) -> bool {
        let (x, y) = (self.root(u), self.root(v));
        self.0[x] = y;
        x != y
    }
}

#[derive(Default)]
struct Level {
    cells: usize,
    splits: usize,
    bispecial: usize,
    forest: usize,
    components_minus_one: usize,
    chords: usize,
    weight_fields: usize,
    support_tests: usize,
    positive_edges: usize,
    log_drop: f64,
}

// Known margins plus support determine a forest. Residual weights on its chords
// determine the remaining entries by peeling leaves, even for disconnected support.
fn matrix(
    mode: &mut Mode,
    start: usize,
    bands: &[Band],
    widths: &[usize],
    stats: &mut Level,
) -> Vec<usize> {
    if bands.len() == 1 {
        return widths.to_vec();
    }
    let (left, right) = (bands.len(), widths.len());
    let mut known = vec![0; left * right];
    if let Mode::Pack { positions, .. } = mode {
        for (i, b) in bands.iter().enumerate() {
            let pos = &positions[b.byte as usize];
            let mut rank = pos.partition_point(|&p| p < start);
            let mut end = start;
            for (j, w) in widths.iter().enumerate() {
                end += w;
                let next = pos.partition_point(|&p| p < end);
                known[i * right + j] = next - rank;
                rank = next;
            }
        }
    }
    let vertices = left + right;
    let mut uf = Forest((0..vertices).collect());
    let mut rem: Vec<usize> = bands
        .iter()
        .map(|b| b.count)
        .chain(widths.iter().copied())
        .collect();
    let mut tree = vec![Vec::new(); vertices];
    let mut chords = Vec::new();
    let mut entries = vec![0; left * right];
    for idx in 0..entries.len() {
        if mode.present(known[idx]) {
            let (u, v) = (idx / right, left + idx % right);
            entries[idx] = 1; // Subtract the positive support before solving the margins.
            rem[u] -= 1;
            rem[v] -= 1;
            if uf.join(u, v) {
                tree[u].push((v, idx));
                tree[v].push((u, idx));
            } else {
                chords.push((u, v, idx));
            }
        }
    }
    let edges = entries.iter().sum::<usize>();
    stats.bispecial += 1;
    stats.forest += usize::from(chords.is_empty());
    stats.components_minus_one += vertices + chords.len() - edges - 1;
    stats.chords += chords.len();
    stats.support_tests += entries.len();
    for (u, v, idx) in chords {
        let limit = rem[u].min(rem[v]);
        stats.weight_fields += usize::from(limit > 0);
        let x = mode.residual(known[idx].saturating_sub(1), limit);
        entries[idx] += x;
        rem[u] -= x;
        rem[v] -= x;
    }
    let mut degree: Vec<_> = tree.iter().map(Vec::len).collect();
    let mut leaves: VecDeque<_> = (0..vertices).filter(|&u| degree[u] == 1).collect();
    while let Some(u) = leaves.pop_front() {
        if degree[u] == 0 {
            continue;
        }
        let &(v, idx) = tree[u].iter().find(|(v, _)| degree[*v] > 0).unwrap();
        let x = rem[u];
        assert!(x <= rem[v]);
        rem[u] = 0;
        rem[v] -= x;
        entries[idx] += x;
        degree[u] = 0;
        degree[v] -= 1;
        if degree[v] == 1 {
            leaves.push_back(v);
        }
    }
    assert!(rem.iter().all(|&x| x == 0));
    if matches!(mode, Mode::Pack { .. }) {
        assert_eq!(entries, known);
    }
    entries
}

fn refine(counts: &[usize; ALPHABET], mode: &mut Mode) -> (Vec<Cell>, Vec<Level>, f64) {
    let n: usize = counts.iter().sum();
    assert!((1..=MAX_BLOCK).contains(&n));
    let mut logfac = vec![0.0; n + 1];
    for i in 1..=n {
        logfac[i] = logfac[i - 1] + (i as f64).log2();
    }
    let initial_log = logfac[n] - counts.iter().map(|&h| logfac[h]).sum::<f64>();
    let mut base = 0;
    let mut root = Vec::new();
    let mut widths = Vec::new();
    for (byte, &count) in counts.iter().enumerate() {
        if count > 0 {
            root.push(Band {
                byte: byte as u8,
                count,
                image: base,
            });
            widths.push(count);
            base += count;
        }
    }
    let mut cells: Vec<_> = (0..n).map(|_| Cell::Empty).collect();
    cells[0] = Cell::new(root);
    let mut current = Vec::new();
    if widths.len() > 1 {
        current.push(Split { start: 0, widths });
    }
    let mut levels = Vec::new();
    let mut cell_count = 1;
    while !current.is_empty() {
        let mut level = Level {
            cells: cell_count,
            ..Level::default()
        };
        let mut next = Vec::new();
        for Split { start, widths } in current {
            level.splits += 1;
            let parent = std::mem::take(&mut cells[start]);
            let bands = parent.bands();
            let size: usize = widths.iter().sum();
            assert_eq!(size, bands.iter().map(|b| b.count).sum::<usize>());
            let m = matrix(mode, start, bands, &widths, &mut level);
            if bands.len() > 1 {
                level.log_drop += logfac[size]
                    - bands.iter().map(|b| logfac[b.count]).sum::<f64>()
                    - widths.iter().map(|&w| logfac[w]).sum::<f64>()
                    + m.iter().map(|&x| logfac[x]).sum::<f64>();
            }
            let mut children = vec![Vec::new(); widths.len()];
            for (b, row) in bands.iter().zip(m.chunks_exact(widths.len())) {
                let mut image = b.image;
                let mut parts = Vec::new();
                for (child, &count) in children.iter_mut().zip(row) {
                    if count > 0 {
                        child.push(Band {
                            byte: b.byte,
                            count,
                            image,
                        });
                        image += count;
                        parts.push(count);
                        level.positive_edges += 1;
                    }
                }
                if parts.len() > 1 {
                    next.push(Split {
                        start: b.image,
                        widths: parts,
                    });
                }
            }
            cell_count += widths.len() - 1;
            let mut offset = start;
            for (child, width) in children.into_iter().zip(widths) {
                assert!(matches!(cells[offset], Cell::Empty));
                cells[offset] = Cell::new(child);
                offset += width;
            }
        }
        levels.push(level);
        current = next;
    }
    assert!(cells.iter().all(|c| !matches!(c, Cell::Mixed(_))));
    (cells, levels, initial_log)
}

fn column(cells: &[Cell]) -> Vec<u8> {
    let mut out = Vec::with_capacity(cells.len());
    for (start, c) in cells.iter().enumerate() {
        if let Cell::One(b) = c {
            assert_eq!(start, out.len());
            out.extend(std::iter::repeat_n(b.byte, b.count));
            assert!(matches!(&cells[b.image], Cell::One(dst) if dst.count == b.count));
        }
    }
    assert_eq!(out.len(), cells.len());
    out
}

fn marked_output(cells: &[Cell], primary: usize) -> Vec<u8> {
    // The offset inside a terminal cell is invariant under LF.
    assert!(primary < cells.len());
    let start = (0..=primary)
        .rev()
        .find(|&p| matches!(cells[p], Cell::One(_)))
        .unwrap();
    let mut cycle = Vec::new();
    let mut at = start;
    loop {
        let Cell::One(b) = &cells[at] else {
            panic!("missing terminal image")
        };
        cycle.push(b.byte);
        at = b.image;
        if at == start {
            break;
        }
        assert!(cycle.len() < cells.len());
    }
    cycle.reverse();
    cycle.iter().copied().cycle().take(cells.len()).collect()
}

fn pack(last: &[u8], primary: usize) -> (Vec<u8>, Vec<Level>, f64, usize) {
    let mut positions = vec![Vec::new(); ALPHABET];
    for (i, &b) in last.iter().enumerate() {
        positions[b as usize].push(i);
    }
    let counts = std::array::from_fn(|i| positions[i].len());
    let mut tape = Tape::default();
    let (cells, levels, log) = refine(
        &counts,
        &mut Mode::Pack {
            positions,
            tape: &mut tape,
        },
    );
    assert_eq!(column(&cells), last);
    let marked_rank = cells[..=primary]
        .iter()
        .filter(|c| !matches!(c, Cell::Empty))
        .count()
        - 1;
    (tape.serialize(&counts), levels, log, marked_rank)
}

fn support_tape(weighted: &[u8], marked_rank: usize) -> Vec<u8> {
    let (counts, tape) = Tape::parse(weighted);
    let mut out = b"SUP1".to_vec();
    out.extend((counts.iter().sum::<usize>() as u32).to_le_bytes());
    out.extend((marked_rank as u32).to_le_bytes());
    let mut mask = [0u8; 32];
    for (a, &count) in counts.iter().enumerate() {
        if count > 0 {
            mask[a / 8] |= 1 << (a % 8);
        }
    }
    out.extend(mask);
    out.extend((tape.bits as u64).to_le_bytes());
    out.extend(tape.support);
    out
}

fn unpack(bytes: &[u8]) -> Vec<Cell> {
    let (counts, tape) = Tape::parse(bytes);
    let mut mode = Mode::Unpack {
        tape: &tape,
        bit: 0,
        weight: 0,
    };
    let (cells, _, _) = refine(&counts, &mut mode);
    if let Mode::Unpack { bit, weight, .. } = mode {
        assert_eq!(bit, tape.bits);
        assert_eq!(weight, tape.weights.len());
    }
    cells
}

fn main() {
    let args: Vec<_> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("pack") if args.len() == 5 => {
            let last = fs::read(&args[2]).unwrap();
            let (data, levels, initial, _) = pack(&last, 0);
            fs::write(&args[3], &data).unwrap();
            let mut csv = std::io::BufWriter::new(fs::File::create(&args[4]).unwrap());
            writeln!(csv, "k,cells,right_special,bispecial,forests,components_minus_one,chords,weight_fields,support_tests,positive_edges,log2_class_drop").unwrap();
            for (k, a) in levels.iter().enumerate() {
                writeln!(
                    csv,
                    "{k},{},{},{},{},{},{},{},{},{},{:.9}",
                    a.cells,
                    a.splits,
                    a.bispecial,
                    a.forest,
                    a.components_minus_one,
                    a.chords,
                    a.weight_fields,
                    a.support_tests,
                    a.positive_edges,
                    a.log_drop
                )
                .unwrap();
            }
            let (_, tape) = Tape::parse(&data);
            println!("root log2 column count {initial:.9}, support bits {}, weight bytes {}, table stream bytes {}", tape.bits, tape.weights.len(), data.len());
        }
        Some("support") if args.len() == 5 => {
            let last = fs::read(&args[2]).unwrap();
            let (weighted, _, _, rank) = pack(&last, args[3].parse().unwrap());
            fs::write(&args[4], support_tape(&weighted, rank)).unwrap();
        }
        Some("unpack") if args.len() == 6 => {
            let cells = unpack(&fs::read(&args[2]).unwrap());
            fs::write(&args[4], column(&cells)).unwrap();
            fs::write(&args[5], marked_output(&cells, args[3].parse().unwrap())).unwrap();
        }
        _ => panic!("pack LAST TAPE TRACE.csv | support LAST PRIMARY SUPPORT_FILE | unpack TAPE PRIMARY LAST_OUT RAW_OUT"),
    }
}

/// Run both sides of the reference transform. The tape alone goes into unpack.
pub fn check_block(last: &[u8], primary: usize) -> (Vec<u8>, [usize; 13]) {
    let (bytes, levels, _, rank) = pack(last, primary);
    let thin = support_tape(&bytes, rank);
    let futures = support::read(&thin);
    let (word, period) = futures.marked_output();
    let distinct = futures.classes();
    if let Some(column) = futures.ordinary_column(period) {
        assert_eq!(column, last);
    }
    let cells = unpack(&bytes);
    assert_eq!(column(&cells), last);
    let (_, tape) = Tape::parse(&bytes);
    let totals = [
        levels.len(),
        levels.iter().map(|l| l.splits).sum(),
        levels.iter().map(|l| l.bispecial).sum(),
        levels.iter().map(|l| l.forest).sum(),
        levels.iter().map(|l| l.components_minus_one).sum(),
        levels.iter().map(|l| l.chords).sum(),
        levels.iter().map(|l| l.weight_fields).sum(),
        levels.iter().map(|l| l.positive_edges).sum(),
        tape.bits,
        tape.weights.len(),
        thin.len(),
        distinct,
        period,
    ];
    let sigma = last
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    assert_eq!(totals[4], totals[5] + sigma - 1);
    assert!(totals[1] <= last.len() - 1 && totals[7] <= 3 * (last.len() - 1));
    assert_eq!(word, marked_output(&cells, primary));
    (word, totals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubling_a_period_preserves_the_unweighted_choices() {
        let encode = |last: &[u8]| {
            let (weighted, _, _, rank) = pack(last, 0);
            support_tape(&weighted, rank)
        };
        let once = encode(b"100");
        let twice = encode(b"110000");
        assert_eq!(&once[8..], &twice[8..]);
        assert_eq!(&once[52..], &[0x67]);
        assert_eq!(
            support::read(&twice).marked_output(),
            (b"001001".to_vec(), 3)
        );
    }

    #[test]
    fn moving_copies_between_orbits() {
        let make = |last: &[u8], p| {
            let (weighted, _, _, rank) = pack(last, p);
            support_tape(&weighted, rank)
        };
        assert_eq!(make(b"aab", 0), make(b"abb", 0));
        assert_ne!(make(b"aab", 1), make(b"abb", 1));
        assert_eq!(support::read(&make(b"aab", 1)).marked_output().0, b"aaa");
        assert_eq!(support::read(&make(b"abb", 1)).marked_output().0, b"bbb");
    }

    #[test]
    fn arbitrary_columns_and_every_mark() {
        let mut columns = 0;
        let mut marked = 0;
        for (alphabet, limit) in [(2usize, 11), (3usize, 7)] {
            for n in 1..=limit {
                for mut value in 0..alphabet.pow(n as u32) {
                    let last: Vec<_> = (0..n)
                        .map(|_| {
                            let b = (value % alphabet) as u8;
                            value /= alphabet;
                            b
                        })
                        .collect();
                    let (bytes, levels, initial, _) = pack(&last, 0);
                    let cells = unpack(&bytes);
                    assert_eq!(column(&cells), last);
                    let drop: f64 = levels.iter().map(|l| l.log_drop).sum();
                    assert!((drop - initial).abs() < 1e-9);
                    let components: usize = levels.iter().map(|l| l.components_minus_one).sum();
                    let chords: usize = levels.iter().map(|l| l.chords).sum();
                    let sigma = last
                        .iter()
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len();
                    assert_eq!(components, chords + sigma - 1);
                    let mut psi: Vec<_> = (0..n).collect();
                    psi.sort_by_key(|&i| last[i]);
                    for p in 0..n {
                        let mut q = p;
                        let reference: Vec<_> = (0..n)
                            .map(|_| {
                                q = psi[q];
                                last[q]
                            })
                            .collect();
                        assert_eq!(marked_output(&cells, p), reference);
                        let rank = cells[..=p]
                            .iter()
                            .filter(|c| !matches!(c, Cell::Empty))
                            .count()
                            - 1;
                        let futures = support::read(&support_tape(&bytes, rank));
                        let (decoded, period) = futures.marked_output();
                        assert_eq!(decoded, reference);
                        if let Some(ordinary) = futures.ordinary_column(period) {
                            assert_eq!(ordinary, last);
                        }
                        marked += 1;
                    }
                    columns += 1;
                }
            }
        }
        println!("{columns} arbitrary byte columns and {marked} marked outputs");
    }
}
