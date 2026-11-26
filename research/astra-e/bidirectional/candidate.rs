//! Open return expressed in retained F coordinates. The endpoint elimination, maximal
//! run splitting and dictionary expansion originate in the 2026-11-18 Open-Return journal.
use crate::bwt::{InverseBwt, LazyCells};
use crate::{runs::Runs, Kernels};
#[path = "../open/endpoint.rs"]
mod endpoint;
mod writer;
#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

#[derive(Clone, Copy)]
struct Band {
    end: u32,    // exclusive bottom rank
    source: u32, // first top rank
    code: u32,
}

#[derive(Default)]
struct Kernel {
    endpoint: endpoint::Endpoint,
    strips: Vec<Strip>,
    image_order: Vec<u32>,
    bands: Vec<Band>,
    top_bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    reverse: Vec<u32>,
    extra: Vec<u32>,
}
// Production records cap a run at 127. Selection is on maximal runs.
// A primary inside a thick strip is a retained singleton. The two fragments
// keep the classification of their parent even if they are short.
fn each_run(records: &[u16], p: u32, limit: u32, mut visit: impl FnMut(u8, u32, u32, bool)) {
    let mut pos = 0;
    let mut i = 0;
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < records.len() && (records[i] >> 8) as u8 == c {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        let end = pos + width;
        if width > limit && (pos..end).contains(&p) {
            if pos < p {
                visit(c, pos, p - pos, false);
            }
            visit(c, p, 1, true);
            if p + 1 < end {
                visit(c, p + 1, end - p - 1, false);
            }
        } else {
            visit(c, pos, width, width <= limit);
        }
        pos = end;
    }
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

impl Kernel {
    fn run<const LIMIT: u32>(&mut self, records: &[u16], n: u32, p: u32, out: &mut Vec<u8>) {
        let mut all = [0u32; 256];
        let mut thin = [0u32; 256];
        let mut thick = [0u32; 256];
        let mut mark = (0usize, 0);
        self.strips.clear();
        each_run(records, p, LIMIT, |byte, x, w, keep| {
            let a = byte as usize;
            if keep {
                if (x..x + w).contains(&p) {
                    mark = (a, thin[a] + (p - x));
                }
                thin[a] += w;
            } else {
                self.strips.push(Strip {
                    from: x,
                    to: all[a],
                    width: w,
                    byte,
                });
                thick[a] += 1;
            }
            all[a] += w;
        });
        let m = thin.iter().sum::<u32>();
        let buckets = starts(thin);
        let mark = buckets[mark.0] + mark.1;
        let first = starts(all);
        let mut write = starts(thick);
        self.image_order.resize(self.strips.len(), 0);
        for (i, strip) in self.strips.iter_mut().enumerate() {
            let a = strip.byte as usize;
            strip.to += first[a];
            self.image_order[write[a] as usize] = i as u32;
            write[a] += 1;
        }
        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        self.induced(n, &thin);
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = 254 + self.offsets.len() as u32 > u32::MAX >> bits;
        self.edges.resize(m as usize, 0);
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
            self.fill::<true, LIMIT>(records, p, buckets, bits);
        } else {
            self.reverse.resize(m as usize, 0);
            self.fill::<false, LIMIT>(records, p, buckets, bits);
        }
        if !wide {
            writer::emit(&self.edges, &self.reverse, &self.spelling, &self.offsets,
                n as usize, mark, bits, out);
            return;
        }
        emit(
            &self.edges,
            &self.extra,
            &self.spelling,
            &self.offsets,
            n as usize,
            mark,
            bits,
            out,
        );
    }

    // A band supplies P^-1(j) = k + j-j0 at successive retained L ranks j.
    // At that row write the inverse edge at S(j), where S is retained stable sort.
    fn fill<const WIDE: bool, const LIMIT: u32>(
        &mut self,
        records: &[u16],
        p: u32,
        mut buckets: [u32; 256],
        bits: u32,
    ) {
        let mut bi = 0;
        let mut j = 0;
        let mut start = 0;
        each_run(records, p, LIMIT, |b, _, w, keep| {
            if !keep {
                return;
            }
            let bucket = &mut buckets[b as usize];
            let mut dst = *bucket;
            *bucket += w;
            let end = j + w;
            while j < end {
                let band = self.bands[bi];
                let limit = end.min(band.end);
                let next = dst + limit - j;
                let source = band.source + j - start;
                let pack = if WIDE { 0 } else { band.code << bits };
                for (entry, value) in self.edges[dst as usize..next as usize]
                    .iter_mut()
                    .zip((source | pack)..)
                {
                    *entry = value;
                }
                if !WIDE {
                    let span = next - dst;
                    for (entry, value) in self.reverse[source as usize..(source + span) as usize]
                        .iter_mut()
                        .zip((dst | pack)..)
                    {
                        *entry = value;
                    }
                }
                if WIDE {
                    self.extra[dst as usize..next as usize].fill(band.code);
                }
                j = limit;
                dst = next;
                if j == band.end {
                    bi += 1;
                    start = j;
                }
            }
        });
    }
}
fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.clear();
    out.reserve(n);
    let mask = (1 << bits) - 1;
    let mut j = mark;
    loop {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        };
        j = edge & mask;
        append_code(code, spelling, offsets, out);
        if j == mark {
            break;
        }
    }
    repeat(n, out);
}

#[inline]
fn append_code(code: u32, spelling: &[u8], offsets: &[u32], out: &mut Vec<u8>) {
    if code < 256 {
        out.push(code as u8);
    } else {
        let w = (code - 256) as usize;
        out.extend_from_slice(&spelling[offsets[w] as usize..offsets[w + 1] as usize]);
    }
}
fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

#[derive(Default)]
pub struct Candidate {
    open: Kernel,
    lazy: LazyCells,
}

impl InverseBwt for Candidate {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(ll, counts, p, out)
    }
    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy
            .invert_checkpointed(ll, counts, checkpoints, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(ll, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if r.records().len() * 20 < n * 3 {
            self.open.run::<16>(r.records(), n as u32, p, out);
            true
        } else {
            self.lazy.invert_runs(r, n, p, out, k)
        }
    }
}


#[cfg(test)]
mod verify {
    use super::Kernel;

    // Inverse stable sort gives the next row in text order, even on a column
    // with unrelated cycles. The current output letter is the next row's L.
    fn oracle(l: &[u8], p: usize) -> Vec<u8> {
        let mut inverse: Vec<_> = (0..l.len()).collect();
        inverse.sort_by_key(|&j| l[j]);
        let mut j = p;
        (0..l.len()).map(|_| {
            j = inverse[j];
            l[j]
        }).collect()
    }

    #[test]
    fn exhaustive_conjugacy() {
        let mut k = Kernel::default();
        let mut out = Vec::new();
        let mut cases = 0;
        for n in 1..=10 {
            for mask in 0..1u32 << n {
                let l: Vec<_> = (0..n).map(|j| ((mask>>j)&1) as u8).collect();
                let records: Vec<u16> = l.iter().map(|&b| ((b as u16)<<8)|1).collect();
                for p in 0..n {
                    let expected = oracle(&l,p);
                    for limit in 1..=3 {
                        match limit {
                            1 => k.run::<1>(&records, n as u32, p as u32, &mut out),
                            2 => k.run::<2>(&records, n as u32, p as u32, &mut out),
                            _ => k.run::<3>(&records, n as u32, p as u32, &mut out),
                        }
                        assert_eq!(out,expected,"column={l:?}, mark={p}, limit={limit}");
                        cases+=1;
                    }
                }
            }
        }
        eprintln!("checked {cases} (column, mark, width) triples");
    }
}
