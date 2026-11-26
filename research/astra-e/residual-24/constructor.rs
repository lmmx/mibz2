//! First return to a union of whole LF strips, with one marked row retained.
use crate::bwt::{InverseBwt, Segmented};
use crate::{runs::Runs, Kernels};

#[derive(Default, Clone, Copy, Debug)]
pub struct Counts {
    pub open_nodes: usize,
    pub steps: usize,
    pub rules: usize,
    pub n: usize,
    pub runs: usize,
    pub retained: usize,
    pub thick: usize,
    pub bands: usize,
    pub words: usize,
    pub copies: usize,
    pub traced: usize,
    pub thick_area: usize,
    pub unit_work: usize,
    pub longest: usize,
    pub table_bytes: usize,
}

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

#[derive(Clone, Copy)]
struct Tile {
    end: u32,
    delta: u32,
    // 0..255 spells one byte. 256 exits to the retained ranks.
    symbol: u16,
}

#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: u32,
    code: u32,
}

#[derive(Default)]
pub struct Kernel {
    endpoint: endpoint::Endpoint,
    narrow: narrow::Endpoint,
    queues: Vec<Vec<u32>>,
    segments: segments::Segments,
    strips: Vec<Strip>,
    image_order: Vec<u32>,
    tiles: Vec<Tile>,
    directory: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    pack: Vec<[u8;3]>,
    headers: Vec<u16>,
    high: Vec<u8>,
    packed:bool,
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

const PAGE: usize = 64;
const TERMINAL: u16 = 256;

impl Kernel {
    // Trace the leftmost uncovered row of an interval in f(S). Its least
    // right slack is the entire bundle certified by this itinerary.
    fn trace(&mut self, mut x0: u32, end: u32, c: u8, k: &mut u32, stats: &mut Counts) {
        while x0 < end {
            let mut width = end - x0;
            let mut x = x0;
            let start = self.spelling.len();
            let exit = loop {
                let mut q = self.directory[x as usize / PAGE] as usize;
                while x >= self.tiles[q].end {
                    q += 1;
                }
                let t = self.tiles[q];
                width = width.min(t.end - x);
                x = x.wrapping_add(t.delta);
                if t.symbol == TERMINAL {
                    break x;
                }
                self.spelling.push(t.symbol as u8);
            };
            let height = self.spelling.len() - start;
            let code = if height == 0 {
                u32::from(c)
            } else {
                stats.words += 1;
                stats.copies += width as usize;
                stats.traced += height;
                stats.thick_area += width as usize * height;
                stats.unit_work += usize::from(width == 1) * height;
                stats.longest = stats.longest.max(height);
                self.spelling[start..].reverse();
                self.spelling.push(c);
                self.offsets.push(self.spelling.len() as u32);
                254 + self.offsets.len() as u32
            };
            self.bands.push(Band {
                end: *k + width,
                delta: exit.wrapping_sub(*k),
                code,
            });
            *k += width;
            x0 += width;
        }
    }

    fn construct<const INDUCE: bool, const QUEUE: bool, const PACK_PAGE:usize>(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
    ) -> (Counts, u32, u32) {
        let mut stats = Counts {
            n: n as usize,
            ..Counts::default()
        };
        let mut all = [0; 256];
        let mut thin = [0; 256];
        let mut thick = [0; 256];
        let mut m = 0;
        let mut mark = 0;
        self.strips.clear();
        if QUEUE {
            self.queues.resize_with(256, Vec::new);
            for q in &mut self.queues {
                q.clear();
            }
        }
        let mut last = None;
        each_run(records, p, limit, |c, x, w, keep| {
            stats.runs += usize::from(last != Some(c));
            last = Some(c);
            let a = usize::from(c);
            if keep {
                if (x..x + w).contains(&p) {
                    mark = m + (p - x);
                }
                if QUEUE {
                    self.queues[a].extend(m..m + w);
                }
                thin[a] += w;
                m += w;
            } else {
                self.strips.push(Strip {
                    from: x,
                    to: all[a],
                    width: w,
                    byte: c,
                });
                thick[a] += 1;
            }
            all[a] += w;
        });
        assert!(m > 0 && all.iter().sum::<u32>() == n);
        stats.retained = m as usize;
        stats.thick = self.strips.len();
        let first = starts(all);
        let mut write = starts(thick);
        self.image_order.resize(self.strips.len(), 0);
        self.tiles.clear();
        let (mut last_x, mut retained_x) = (0, 0u32);
        for (i, s) in self.strips.iter_mut().enumerate() {
            let a = usize::from(s.byte);
            s.to += first[a];
            self.image_order[write[a] as usize] = i as u32;
            write[a] += 1;
            if !INDUCE {
                if s.from > last_x {
                    self.tiles.push(Tile {
                        end: s.from,
                        delta: retained_x.wrapping_sub(last_x),
                        symbol: TERMINAL,
                    });
                    retained_x += s.from - last_x;
                }
                last_x = s.from + s.width;
                self.tiles.push(Tile {
                    end: last_x,
                    delta: s.to.wrapping_sub(s.from),
                    symbol: s.byte.into(),
                });
            }
        }
        if !INDUCE {
            if last_x < n {
                self.tiles.push(Tile {
                    end: n,
                    delta: retained_x.wrapping_sub(last_x),
                    symbol: TERMINAL,
                });
            }
            self.directory.clear();
            let mut ti = 0;
            for x in (0..n).step_by(PAGE) {
                while self.tiles[ti].end <= x {
                    ti += 1;
                }
                self.directory.push(ti as u32);
            }
        }
        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        let mut cursors = if INDUCE {
            // H strips plus at most H+1 gap nodes on each side, plus the sentinel.
            if self.strips.len() <= (u16::MAX as usize - 3) / 3 {
                self.induced_narrow(n, &thin, &mut stats)
            } else {
                self.induced(n, &thin, &mut stats)
            }
        } else {
            let mut cursors = [0usize; 256];
            let (mut k, mut si) = (0, 0);
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let mut x = first[a];
                while si < write[a] as usize {
                    let s = self.strips[self.image_order[si] as usize];
                    self.trace(x, s.to, a as u8, &mut k, &mut stats);
                    x = s.to + s.width;
                    si += 1;
                }
                self.trace(x, first[a] + all[a], a as u8, &mut k, &mut stats);
            }
            assert_eq!(k, m);
            cursors
        };
        stats.bands = self.bands.len();
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
        self.packed=false;
        if PACK_PAGE>0 && QUEUE && !wide && self.build_packed::<PACK_PAGE>(m as usize,bits,&cursors) {
            self.packed=true;
            stats.table_bytes=self.pack.len()*3+self.headers.len()*2+self.high.len();
            return (stats,mark,bits)
        }
        self.edges.resize(m as usize, 0);
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        stats.table_bytes = 4 * (self.edges.len() + self.extra.len());
        if QUEUE {
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut from = 0;
                for b in &self.bands[cursors[a]..end] {
                    let span = (b.end - k) as usize;
                    let dst = k.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(b.code);
                    }
                    let src = &self.queues[a][from..from + span];
                    for (target, &j) in self.edges[dst..dst + span].iter_mut().zip(src) {
                        *target = pack | j;
                    }
                    from += span;
                    k = b.end;
                }
            }
        } else {
            let mut stream = starts(thin);
            let mut j = 0;
            each_run(records, p, limit, |c, _, w, keep| {
                if !keep {
                    return;
                }
                let a = usize::from(c);
                let mut z = stream[a];
                let end = z + w;
                while z < end {
                    let b = self.bands[cursors[a]];
                    let span = (end.min(b.end) - z) as usize;
                    let dest = z.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dest..dest + span].fill(b.code);
                    }
                    let mut value = pack | j;
                    for e in &mut self.edges[dest..dest + span] {
                        *e = value;
                        value = value.wrapping_add(1);
                    }
                    j += span as u32;
                    z += span as u32;
                    cursors[a] += usize::from(z == b.end);
                }
                stream[a] = end;
            });
        }
        (stats, mark, bits)
    }

    pub fn run<const INDUCE: bool, const QUEUE: bool, const SEGMENTS: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> Counts {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let (stats, mark, bits) = self.construct::<INDUCE, QUEUE,0>(records, n as u32, p, limit);
        if SEGMENTS {
            self.segments.emit(
                &self.edges,
                &self.extra,
                &self.spelling,
                &self.offsets,
                n,
                mark,
                bits,
                out,
            );
        } else {
            emit(
                &self.edges,
                &self.extra,
                &self.spelling,
                &self.offsets,
                n,
                mark,
                bits,
                out,
            );
        }
        stats
    }

    pub fn price<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        runs: &Runs,
        n: usize,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> (f64, f64) {
        let begin = std::time::Instant::now();
        let (_, mark, bits) = self.construct::<INDUCE, QUEUE,0>(runs.records(), n as u32, p, limit);
        let built = begin.elapsed().as_secs_f64();
        let start = std::time::Instant::now();
        emit(
            &self.edges,
            &self.extra,
            &self.spelling,
            &self.offsets,
            n,
            mark,
            bits,
            out,
        );
        (built, start.elapsed().as_secs_f64())
    }

    pub fn column<const INDUCE: bool>(
        &mut self,
        l: &[u8],
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> Counts {
        self.run::<INDUCE, false, false>(&records_of(l), l.len(), p, limit, out)
    }
}

#[derive(Default)]
pub struct Open<
    const LIMIT: u32,
    const INDUCE: bool = false,
    const QUEUE: bool = false,
    const SEGMENTS: bool = false,
> {
    kernel: Kernel,
    other: Segmented,
}

impl<const LIMIT: u32, const INDUCE: bool, const QUEUE: bool, const SEGMENTS: bool> InverseBwt
    for Open<LIMIT, INDUCE, QUEUE, SEGMENTS>
{
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, _: Kernels) -> bool {
        self.kernel
            .run::<INDUCE, QUEUE, SEGMENTS>(r.records(), n, p, LIMIT, out);
        true
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.other.invert(l, c, p, out);
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.other.invert_in_place(l, c, p)
    }
}

fn emit(
    edges: &[u32], extra: &[u32], spelling: &[u8], offsets: &[u32],
    n: usize, mark: u32, bits: u32, out: &mut Vec<u8>,
) {
    out.clear();
    out.reserve(n);
    let mask = (1 << bits) - 1;
    let mut row = mark;
    let mut edge = edges[row as usize];
    loop {
        let code = if extra.is_empty() { edge >> bits } else { extra[row as usize] };
        row = edge & mask;
        // The marked cycle always has a successor, including its closing edge.
        // Fetch that successor while the current word is still available.
        edge = edges[row as usize];
        if code < 256 {
            out.push(code as u8);
        } else {
            let w = (code - 256) as usize;
            out.extend_from_slice(&spelling[offsets[w] as usize..offsets[w + 1] as usize]);
        }
        if row == mark {
            break;
        }
    }
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
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

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

#[path = "../short-words/endpoint.rs"]
mod endpoint;
#[path = "narrow.rs"]
mod narrow;
#[path = "../../../docs/journal/2026-11-18-open-return/segments.rs"]
mod segments;


impl Kernel {
    fn build_packed<const P:usize>(&mut self,m:usize,bits:u32,cursors:&[usize;256])->bool {
        debug_assert!(P.is_power_of_two());
        const MIXED:u16=u16::MAX;
        const CLEAR:u16=u16::MAX-1;
        let low=24-bits;
        let largest=255+self.offsets.len().saturating_sub(1);
        if largest>>(low as usize)>255 {return false}
        self.headers.resize(m.div_ceil(P),CLEAR);
        self.headers.fill(CLEAR);
        let mut k=0;
        for b in &self.bands {
            let span=(b.end-k) as usize;
            let dst=k.wrapping_add(b.delta) as usize;
            let high=(b.code>>low) as u16;
            for h in &mut self.headers[dst/P..=(dst+span-1)/P] {
                if *h==CLEAR {*h=high;} else if *h!=high {*h=MIXED;}
            }
            k=b.end;
        }
        let mut mixed=0usize;
        for h in &mut self.headers {
            if *h==MIXED {
                if mixed>=32768 {return false}
                *h=0x8000 | mixed as u16;
                mixed+=1;
            } else {debug_assert!(*h<=255);}
        }
        self.pack.resize(m+1,[0;3]);
        self.high.resize(mixed*P,0);
        let code_mask=(1<<low)-1;
        let mut k=0;
        for a in 0..256 {
            let end=cursors.get(a+1).copied().unwrap_or(self.bands.len());
            let mut from=0;
            for b in &self.bands[cursors[a]..end] {
                let span=(b.end-k) as usize;
                let dst=k.wrapping_add(b.delta) as usize;
                let pack=(b.code&code_mask)<<bits;
                let top=(b.code>>low) as u8;
                let src=&self.queues[a][from..from+span];
                for (slot,&j) in self.pack[dst..dst+span].iter_mut().zip(src) {
                    let [x,y,z,_]=(pack|j).to_le_bytes();
                    *slot=[x,y,z];
                }
                let limit=dst+span;
                let mut next=dst;
                while next<limit {
                    let h=self.headers[next/P];
                    let chunk=(P-next%P).min(limit-next);
                    if h&0x8000!=0 {
                        let offset=(h as usize & 0x7fff)*P+next%P;
                        self.high[offset..offset+chunk].fill(top);
                    }
                    next+=chunk;
                }
                from+=span;
                k=b.end;
            }
        }
        debug_assert_eq!(k as usize,m);
        true
    }

    #[inline(always)]
    fn load24(&self,row:u32)->u32 {
        let i=row as usize;
        assert!(i+1<self.pack.len());
        // SAFETY: every live row has three initialized bytes and the padding
        // record provides a fourth byte even at the final actual row.
        let p=self.pack.as_ptr().cast::<u8>();
        unsafe {u32::from_le(p.add(3*i).cast::<u32>().read_unaligned()) & 0x00ff_ffff}
    }
    #[inline(always)]
    fn label24<const P:usize>(&self,row:u32,edge:u32,bits:u32)->u32 {
        let i=row as usize;
        let header=self.headers[i/P];
        let high=if header&0x8000==0 {u32::from(header)} else {
            u32::from(self.high[(usize::from(header)&0x7fff)*P+i%P])
        };
        (high<<(24-bits))|(edge>>bits)
    }
    fn emit_packed<const P:usize>(&self,n:usize,mark:u32,bits:u32,out:&mut Vec<u8>) {
        out.clear();out.reserve(n);
        let mask=(1<<bits)-1;
        let mut row=mark;
        let mut edge=self.load24(row);
        loop {
            let current=row;
            row=edge & mask;
            // The word metadata and next link are independent after row is known.
            let next=self.load24(row);
            let code=self.label24::<P>(current,edge,bits);
            append_code(code,&self.spelling,&self.offsets,out);
            edge=next;
            if row==mark {break}
        }
        repeat(n,out);
    }
    pub fn packed_run<const P:usize>(&mut self, records:&[u16], n:usize, p:u32, out:&mut Vec<u8>) {
        let (_,mark,bits)=self.construct::<true,true,P>(records,n as u32,p,16);
        if self.packed {
            self.emit_packed::<P>(n,mark,bits,out)
        } else {
            emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
        }
    }
}
