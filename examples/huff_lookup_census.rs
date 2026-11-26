//! (research only, `explore`) How many dependent table lookups the scanner's Huffman decode
//! needs a symbol, on a real file, under the shipped root table and under wider ones.
//! docs/journal/2026-11-26-explore/scan-lookups-and-runts.md.
//!
//!     huff_lookup_census FILE.bz2
//!
//! A standalone reader of the format (it shares no code with the decoder): each block's
//! header, bitmap, selectors and code lengths, then its symbols decoded bit by bit, as
//! libbzip2's `GET_MTF_VAL` does. For every 50-symbol group it keeps the code lengths in
//! order and counts the lookups a root table of `B` bits needs that yields up to `K`
//! whole codes an entry (greedy, within the group: what such a table's entry holds), a
//! code longer than `B` taking two (root, then second level). `B = 10, K = 2` is the
//! shipped table (`src/huff.rs`: a 10-bit root, pairs within a group, a second level past
//! 10 bits). Also: the code-length histogram, the share of RUNA/RUNB, and the blocks' sizes.
//! Untimed: lookups are a count, not a time.

const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
const END_MAGIC: u64 = 0x1772_4538_5090;

struct Bits<'a> {
    d: &'a [u8],
    pos: usize,
}

impl Bits<'_> {
    fn bit(&mut self) -> u32 {
        let b = (self.d[self.pos >> 3] >> (7 - (self.pos & 7))) & 1;
        self.pos += 1;
        b as u32
    }
    fn bits(&mut self, n: u32) -> u64 {
        let mut v = 0u64;
        for _ in 0..n {
            v = v << 1 | self.bit() as u64;
        }
        v
    }
}

/// A table's canonical decode: libbzip2's limit, base and perm, and its shortest length
type Canon = (Vec<i32>, Vec<i32>, Vec<u16>, u32);

const BS: [u32; 4] = [10, 11, 12, 13];
const KS: [usize; 4] = [1, 2, 3, 4];

#[derive(Default)]
struct Tally {
    syms: u64,
    bits: u64,
    run: u64,
    hist: [u64; 21],
    lookups: [[u64; 4]; 4],
    /// lookups with pairs allowed to cross a group boundary, `B = 10, K = 2`
    cross: u64,
}

/// The shipped table's lookups (`B = 10, K = 2`): each one's 10-bit index, by table
fn shipped_indices(lens: &[u8], starts: &[usize], d: &[u8], hist: &mut [u64; 1024]) {
    let mut i = 0;
    while i < lens.len() {
        let p = starts[i];
        let w = u32::from_be_bytes([
            d[p >> 3],
            *d.get((p >> 3) + 1).unwrap_or(&0),
            *d.get((p >> 3) + 2).unwrap_or(&0),
            0,
        ]);
        hist[((w << (p & 7)) >> 22) as usize] += 1;
        if lens[i] > 10 {
            i += 1;
        } else if i + 1 < lens.len() && lens[i] + lens[i + 1] <= 10 {
            i += 2;
        } else {
            i += 1;
        }
    }
}

fn count_group(lens: &[u8], t: &mut Tally) {
    for (bi, &b) in BS.iter().enumerate() {
        for (ki, &k) in KS.iter().enumerate() {
            let mut i = 0;
            let mut n = 0u64;
            while i < lens.len() {
                if lens[i] as u32 > b {
                    n += 2;
                    i += 1;
                    continue;
                }
                let (mut used, mut take) = (0u32, 0);
                while take < k && i + take < lens.len() && used + lens[i + take] as u32 <= b {
                    used += lens[i + take] as u32;
                    take += 1;
                }
                n += 1;
                i += take;
            }
            t.lookups[bi][ki] += n;
        }
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: huff_lookup_census FILE.bz2");
    let d = std::fs::read(path).expect("read");
    let mut br = Bits { d: &d, pos: 0 };
    let mut t = Tally::default();
    let mut all_lens: Vec<u8> = Vec::new();
    let mut starts: Vec<usize> = Vec::new();
    // per full block and table: how many of the 1,024 root entries take 50% / 90% / 99% of its lookups
    let mut cover: Vec<[usize; 3]> = Vec::new();
    let (mut blocks, mut runts, mut runt_syms) = (0u64, 0u64, 0u64);
    let mut groups_hist = [0u64; 7];
    while br.pos + 32 <= d.len() * 8 {
        assert_eq!(br.bits(24), 0x425a68, "stream header");
        let _level = br.bits(8);
        loop {
            let magic = br.bits(48);
            if magic == END_MAGIC {
                br.bits(32);
                br.pos = br.pos.div_ceil(8) * 8;
                break;
            }
            assert_eq!(magic, BLOCK_MAGIC);
            br.bits(32 + 1 + 24);
            let used16 = br.bits(16);
            let mut in_use = 0;
            for i in 0..16 {
                if used16 >> (15 - i) & 1 == 1 {
                    in_use += br.bits(16).count_ones();
                }
            }
            let alpha = in_use as usize + 2;
            let n_groups = br.bits(3) as usize;
            groups_hist[n_groups] += 1;
            let n_sel = br.bits(15) as usize;
            let mut mtf: Vec<u8> = (0..n_groups as u8).collect();
            let mut sel = Vec::with_capacity(n_sel);
            for _ in 0..n_sel {
                let mut j = 0;
                while br.bit() == 1 {
                    j += 1;
                }
                let v = mtf.remove(j);
                mtf.insert(0, v);
                sel.push(v);
            }
            let mut lens = vec![vec![0u8; alpha]; n_groups];
            for l in lens.iter_mut() {
                let mut c = br.bits(5) as i32;
                for x in l.iter_mut() {
                    while br.bit() == 1 {
                        c += if br.bit() == 0 { 1 } else { -1 };
                    }
                    *x = c as u8;
                }
            }
            // canonical decode tables, libbzip2's (limit, base, perm)
            let tabs: Vec<Canon> = lens
                .iter()
                .map(|l| {
                    let (mn, mx) = (
                        *l.iter().min().unwrap() as usize,
                        *l.iter().max().unwrap() as usize,
                    );
                    let mut perm = Vec::new();
                    for len in mn..=mx {
                        for (s, &x) in l.iter().enumerate() {
                            if x as usize == len {
                                perm.push(s as u16);
                            }
                        }
                    }
                    let mut base = vec![0i32; 23];
                    for &x in l {
                        base[x as usize + 1] += 1;
                    }
                    for i in 1..23 {
                        base[i] += base[i - 1];
                    }
                    let mut limit = vec![0i32; 23];
                    let mut vec = 0i32;
                    for i in mn..=mx {
                        vec += base[i + 1] - base[i];
                        limit[i] = vec - 1;
                        vec <<= 1;
                    }
                    for i in mn + 1..=mx {
                        base[i] = ((limit[i - 1] + 1) << 1) - base[i];
                    }
                    (limit, base, perm, mn as u32)
                })
                .collect();
            let eob = (alpha - 1) as u16;
            let mut ih = vec![[0u64; 1024]; n_groups];
            let mut nsyms = 0u64;
            'g: for &g in &sel {
                let (limit, base, perm, mn) = &tabs[g as usize];
                let gl = &lens[g as usize];
                all_lens.clear();
                starts.clear();
                for _ in 0..50 {
                    starts.push(br.pos);
                    let mut n = *mn;
                    let mut v = br.bits(n) as i32;
                    while v > limit[n as usize] {
                        n += 1;
                        v = v << 1 | br.bit() as i32;
                    }
                    let s = perm[(v - base[n as usize]) as usize];
                    let len = gl[s as usize];
                    all_lens.push(len);
                    t.syms += 1;
                    nsyms += 1;
                    t.bits += len as u64;
                    t.hist[len as usize] += 1;
                    if s < 2 {
                        t.run += 1;
                    }
                    if s == eob {
                        shipped_indices(&all_lens, &starts, &d, &mut ih[g as usize]);
                        count_group(&all_lens, &mut t);
                        break 'g;
                    }
                }
                shipped_indices(&all_lens, &starts, &d, &mut ih[g as usize]);
                count_group(&all_lens, &mut t);
            }
            if nsyms >= 100_000 {
                for h in &ih {
                    let mut v: Vec<u64> = h.to_vec();
                    v.sort_unstable_by(|a, b| b.cmp(a));
                    let tot: u64 = v.iter().sum();
                    let mut c = [0usize; 3];
                    for (q, f) in [0.5, 0.9, 0.99].iter().enumerate() {
                        let (mut acc, mut k) = (0u64, 0);
                        while (acc as f64) < f * tot as f64 {
                            acc += v[k];
                            k += 1;
                        }
                        c[q] = k;
                    }
                    cover.push(c);
                }
            }
            blocks += 1;
            if nsyms < 1000 {
                runts += 1;
                runt_syms += nsyms;
            }
        }
    }
    let s = t.syms as f64;
    println!("blocks {blocks} (runts under 1,000 symbols: {runts}, {runt_syms} symbols)");
    println!("tables a block (2..6): {:?}", &groups_hist[2..]);
    println!(
        "symbols {} · mean code length {:.3} bits · RUNA/RUNB {:.1}%",
        t.syms,
        t.bits as f64 / s,
        100.0 * t.run as f64 / s
    );
    let mut cum = 0u64;
    print!("code length: share (cumulative)");
    for (l, &h) in t.hist.iter().enumerate() {
        if h > 0 {
            cum += h;
            print!(
                " | {l}: {:.2}% ({:.2}%)",
                100.0 * h as f64 / s,
                100.0 * cum as f64 / s
            );
        }
    }
    println!();
    println!("lookups a symbol (greedy within a 50-symbol group; > B bits takes two):");
    println!("  B \\ K     1      2      3      4");
    for (bi, b) in BS.iter().enumerate() {
        print!("  {b:>2}   ");
        for ki in 0..4 {
            print!(" {:.4}", t.lookups[bi][ki] as f64 / s);
        }
        println!();
    }
    let _ = t.cross;
    for (q, f) in ["50%", "90%", "99%"].iter().enumerate() {
        let mut v: Vec<usize> = cover.iter().map(|c| c[q]).collect();
        v.sort_unstable();
        println!(
            "shipped root table, entries taking {f} of a table's lookups (blocks of 100k+ symbols, {} tables): p10 {} · median {} · p90 {} of 1024",
            v.len(), v[v.len() / 10], v[v.len() / 2], v[v.len() * 9 / 10]
        );
    }
}
