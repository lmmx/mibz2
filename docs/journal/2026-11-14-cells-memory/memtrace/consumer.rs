//! The memtrace tool's consumer: reads its records (mt_main.c's format) from a FIFO and
//! writes one JSON line per traced kernel attempt (src/memtrace.rs's markers).
//!
//!     rustc -O -o consumer consumer.rs && consumer TRACE_FIFO OUT.jsonl
//!
//! Per attempt, for every (phase, region, load or store): bytes, accesses, distinct 64-byte
//! lines, and the LRU stack distance of every line access (the number of distinct lines
//! touched since that line's last access in the same attempt, in power-of-two buckets; a
//! line's first access in the attempt is "cold"). An access is split over the lines it
//! covers; a line access that repeats the line just accessed has distance 0 and does not
//! advance the clock. Per region: distinct lines, lines touched in more than one phase,
//! the lines' phase patterns, and 4 KiB pages touched for the first time in this process.
//! Region of an access: by its first byte, among the attempt's REGION markers' [base,
//! base + capacity); within 1 MiB of the END marker's stack address, "stack" (99); else
//! "other" (0).

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::hash::{BuildHasherDefault, Hasher};
use std::io::{BufReader, BufWriter, Read, Write};

#[derive(Default)]
struct H(u64);
impl Hasher for H {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = (self.0.rotate_left(8) ^ x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        }
    }
    fn write_u64(&mut self, x: u64) {
        self.0 = (self.0 ^ x ^ (x >> 29)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
}
type Map<K, V> = HashMap<K, V, BuildHasherDefault<H>>;

const PHASES: [u64; 15] = [0, 10, 11, 12, 20, 30, 31, 32, 33, 34, 35, 36, 50, 51, 52];
const BUCKETS: usize = 26; // 0..=24: floor(log2(d + 1)), 25: cold

fn phase_index(p: u64) -> u8 {
    PHASES.iter().position(|&x| x == p).expect("a known phase") as u8
}

#[derive(Clone)]
struct Stat {
    bytes: u64,
    accesses: u64,
    lines: u64,
    reuse: [u64; BUCKETS],
}
impl Default for Stat {
    fn default() -> Self {
        Stat { bytes: 0, accesses: 0, lines: 0, reuse: [0; BUCKETS] }
    }
}

struct Line {
    region: u8,
    rmask: u16,
    wmask: u16,
    last: u32,
}

struct Fenwick(Vec<u32>);
impl Fenwick {
    fn add(&mut self, i: usize, v: i32) {
        let mut i = i + 1;
        while i < self.0.len() {
            self.0[i] = (self.0[i] as i64 + v as i64) as u32;
            i += i & i.wrapping_neg();
        }
    }
    /// sum of [0, i)
    fn prefix(&self, i: usize) -> u64 {
        let (mut i, mut s) = (i, 0u64);
        while i > 0 {
            s += self.0[i] as u64;
            i -= i & i.wrapping_neg();
        }
        s
    }
}

#[derive(Default)]
struct Attempt {
    head: [u64; 4],
    events: Vec<u64>,
    phases: Vec<u8>,
    regions: Vec<[u64; 4]>,
    end: [u64; 4],
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut inp = BufReader::with_capacity(1 << 22, std::fs::File::open(&args[1]).unwrap());
    let mut out = BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    let mut pages: HashSet<u64, BuildHasherDefault<H>> = HashSet::default();
    let mut cur: Option<Attempt> = None;
    let mut phase = 0u8;
    let mut stray = 0u64;
    let mut attempts = 0u64;
    let mut word = || -> Option<u64> {
        let mut b = [0u8; 8];
        match inp.read_exact(&mut b) {
            Ok(()) => Some(u64::from_le_bytes(b)),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => None,
            Err(e) => panic!("{e}"),
        }
    };
    while let Some(w) = word() {
        let kind = w >> 62;
        if kind == 2 {
            let code = w & 0xFFFF_FFFF;
            let mut p = [0u64; 4];
            for x in &mut p {
                *x = word().expect("a marker cut short");
            }
            match code {
                1 => {
                    assert!(cur.is_none(), "BEGIN inside an attempt");
                    cur = Some(Attempt { head: p, ..Default::default() });
                    phase = 0;
                }
                2 => phase = phase_index(p[0]),
                3 => cur.as_mut().expect("REGION outside an attempt").regions.push(p),
                4 => {
                    let mut a = cur.take().expect("END outside an attempt");
                    a.end = p;
                    let line = process(&a, &mut pages);
                    out.write_all(line.as_bytes()).unwrap();
                    out.write_all(b"\n").unwrap();
                    attempts += 1;
                }
                _ => panic!("unknown marker {code}"),
            }
        } else if let Some(a) = cur.as_mut() {
            a.events.push(w);
            a.phases.push(phase);
        } else {
            stray += 1;
        }
    }
    out.flush().unwrap();
    eprintln!("consumer: {attempts} attempts, {stray} accesses outside attempts");
}

fn process(a: &Attempt, pages: &mut HashSet<u64, BuildHasherDefault<H>>) -> String {
    // regions by base, for the lookup
    let mut iv: Vec<(u64, u64, u8)> = a
        .regions
        .iter()
        .filter(|r| r[1] != 0 && r[3] > 0)
        .map(|r| (r[1], r[1] + r[3], r[0] as u8))
        .collect();
    iv.sort_unstable();
    let sp = a.end[3];
    let region_of = |x: u64| -> u8 {
        let i = iv.partition_point(|r| r.0 <= x);
        if i > 0 && x < iv[i - 1].1 {
            return iv[i - 1].2;
        }
        if x.abs_diff(sp) < 1 << 20 {
            99
        } else {
            0
        }
    };
    let mut lines: Map<u64, Line> = Map::default();
    let mut stats: Map<(u8, u8, u8), Stat> = Map::default();
    let mut new_pages: Map<u8, u64> = Map::default();
    let total: u64 = a
        .events
        .iter()
        .map(|&w| {
            let (addr, size) = (w & 0xFFFF_FFFF_FFFF, ((w >> 48) & 0x3FFF).max(1));
            ((addr + size - 1) >> 6) - (addr >> 6) + 1
        })
        .sum();
    let mut fw = Fenwick(vec![0; total as usize + 2]);
    let mut t = 0usize;
    let mut prev_line = u64::MAX;
    let mut line_events = 0u64;
    for (i, &w) in a.events.iter().enumerate() {
        let addr = w & 0xFFFF_FFFF_FFFF;
        let size = ((w >> 48) & 0x3FFF).max(1);
        let rw = ((w >> 62) & 1) as u8;
        let ph = a.phases[i];
        let reg = region_of(addr);
        let st = stats.entry((ph, reg, rw)).or_default();
        st.bytes += size;
        st.accesses += 1;
        let (l0, l1) = (addr >> 6, (addr + size - 1) >> 6);
        for l in l0..=l1 {
            line_events += 1;
            let bucket;
            if l == prev_line {
                bucket = 0;
                let e = lines.get_mut(&l).unwrap();
                let bit = 1u16 << ph;
                let m = if rw == 1 { &mut e.wmask } else { &mut e.rmask };
                if *m & bit == 0 {
                    *m |= bit;
                    st.lines += 1;
                }
            } else {
                t += 1;
                match lines.get_mut(&l) {
                    Some(e) => {
                        let last = e.last as usize;
                        let d = fw.prefix(t) - fw.prefix(last + 1);
                        fw.add(last, -1);
                        bucket = (64 - (d + 1).leading_zeros() as usize - 1).min(24);
                        e.last = t as u32;
                        let bit = 1u16 << ph;
                        let m = if rw == 1 { &mut e.wmask } else { &mut e.rmask };
                        if *m & bit == 0 {
                            *m |= bit;
                            st.lines += 1;
                        }
                    }
                    None => {
                        bucket = BUCKETS - 1;
                        let bit = 1u16 << ph;
                        lines.insert(
                            l,
                            Line {
                                region: reg,
                                rmask: if rw == 0 { bit } else { 0 },
                                wmask: if rw == 1 { bit } else { 0 },
                                last: t as u32,
                            },
                        );
                        st.lines += 1;
                        if pages.insert(l >> 6) {
                            *new_pages.entry(reg).or_default() += 1;
                        }
                    }
                }
                fw.add(t, 1);
                prev_line = l;
            }
            st.reuse[bucket] += 1;
        }
    }
    // per region: lines, lines of more than one phase, the phase patterns
    let mut per: Map<u8, (u64, u64, Map<(u16, u16), u64>)> = Map::default();
    for e in lines.values() {
        let x = per.entry(e.region).or_default();
        x.0 += 1;
        x.1 += u64::from((e.rmask | e.wmask).count_ones() > 1);
        *x.2.entry((e.wmask, e.rmask)).or_default() += 1;
    }
    let names = |m: u16| -> String {
        (0..16)
            .filter(|b| m >> b & 1 == 1)
            .map(|b| PHASES[b].to_string())
            .collect::<Vec<_>>()
            .join("+")
    };
    let mut s = String::new();
    write!(
        s,
        "{{\"path\":{},\"block\":{},\"n\":{},\"records\":{},\"outcome\":{},\"period\":{},\"cells\":{},\"events\":{},\"line_events\":{line_events},\"distinct_lines\":{}",
        a.head[0], a.head[1], a.head[2], a.head[3], a.end[0], a.end[1], a.end[2], a.events.len(), lines.len()
    )
    .unwrap();
    s += ",\"regions\":[";
    let mut first = true;
    for r in &a.regions {
        if !first {
            s.push(',');
        }
        first = false;
        write!(s, "[{},{},{}]", r[0], r[2], r[3]).unwrap();
    }
    s += "],\"stats\":[";
    let mut keys: Vec<_> = stats.keys().copied().collect();
    keys.sort_unstable();
    for (i, k) in keys.iter().enumerate() {
        let v = &stats[k];
        if i > 0 {
            s.push(',');
        }
        let reuse: Vec<String> = v.reuse.iter().map(u64::to_string).collect();
        write!(
            s,
            "{{\"phase\":{},\"region\":{},\"rw\":\"{}\",\"bytes\":{},\"accesses\":{},\"lines\":{},\"reuse\":[{}]}}",
            PHASES[k.0 as usize],
            k.1,
            if k.2 == 1 { "w" } else { "r" },
            v.bytes,
            v.accesses,
            v.lines,
            reuse.join(",")
        )
        .unwrap();
    }
    s += "],\"region_lines\":{";
    let mut regs: Vec<_> = per.keys().copied().collect();
    regs.sort_unstable();
    for (i, r) in regs.iter().enumerate() {
        let (n, multi, pats) = &per[r];
        if i > 0 {
            s.push(',');
        }
        let mut pv: Vec<_> = pats.iter().collect();
        pv.sort_by(|a, b| b.1.cmp(a.1));
        let ps: Vec<String> = pv
            .iter()
            .map(|((w, rd), c)| format!("\"W{}|R{}\":{}", names(*w), names(*rd), c))
            .collect();
        write!(
            s,
            "\"{r}\":{{\"lines\":{n},\"multi_phase\":{multi},\"new_pages\":{},\"patterns\":{{{}}}}}",
            new_pages.get(r).copied().unwrap_or(0),
            ps.join(",")
        )
        .unwrap();
    }
    s += "}}";
    s
}
