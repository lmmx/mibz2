//! Quote-ended phrase BWT, with unary suffix paths contracted before materializing nodes.
use crate::bwt::{InverseBwt, Segmented};
use crate::runs::Runs;
use crate::Kernels;

#[path = "../../../f2c/second.rs"]
mod second;

const QUOTE: u8 = b'"';
const NONE: u32 = u32::MAX;

#[repr(C, align(8))]
#[derive(Clone,Copy,Default)]
struct Run {
    position_byte: u32,
    delta: u32,
}
impl Run {
    #[inline]
    fn retires(self,hi:u32)->bool {self.position_byte<(hi<<8) || self.byte()==QUOTE}
    #[inline]
    fn new(pos:u32, byte:u8, delta:u32)->Self {
        Self {position_byte:(pos<<8) | u32::from(byte),delta}
    }
    #[inline]
    fn position(self)->u32 { self.position_byte >> 8 }
    #[inline]
    fn byte(self)->u8 { self.position_byte as u8 }
    #[inline]
    fn map(self, row:u32)->u32 { row.wrapping_add(self.delta) }
}
#[derive(Clone, Copy, Default)]
struct Piece {
    id: u32,
    width: u32,
}
#[derive(Clone, Copy, Default)]
struct Node {
    lo: u32,
    hi: u32,
    parent: u32,
    first: u8,
    edge: u32,
    end: u32,
    sel: u32,
    sels: u32,
    base: u32,
    word: u32,
    out: u32,
    cursor: u32,
    used: u32,
}
impl Node {
    fn new(lo: u32, parent: u32, byte: u8) -> Self {
        Self {
            lo,
            hi: lo,
            parent,
            first: byte,
            word: NONE,
            ..Self::default()
        }
    }
}

pub type Candidate = Dispatch<2>;

#[derive(Default)]
pub struct Dispatch<const MODE:u8,const SCHEDULE:u8=1> {
    #[cfg(slot_counts)]
    wave_counts:[[u64;9];9],
    fallback: Segmented,
    second: second::Second,
    lane_paths: [Vec<u8>; 8],
    #[cfg(test)]
    histogram: [u64; 9],
    runs: Vec<Run>,
    page_first: Vec<u32>,
    starts: Vec<u64>,
    nodes: Vec<Node>,
    selectors: Vec<Piece>,
    edges: Vec<u8>,
    words: Vec<Piece>,
    dict: Vec<u8>,
    merged: Vec<Piece>,
    links: Vec<u32>,
    #[cfg(test)]
    stats: [u64; 8],
}

impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    fn prepare(&mut self, caps: &[u16], n: u32) -> Option<(u32, u32)> {
        let mut counts=[0u32;256];
        for &record in caps {
            counts[(record>>8) as usize]+=u32::from(record&255);
        }
        self.prepare_body(caps,n,&counts)
    }
    #[inline]
    fn locate(&self, x: u32) -> usize {
        let page = (x >> 6) as usize;
        let mask = u64::MAX >> (63 - (x & 63));
        (self.page_first[page] + (self.starts[page] & mask).count_ones()) as usize
    }
    #[inline(always)]
    fn prepare_body(&mut self, caps: &[u16], n: u32, counts: &[u32; 256]) -> Option<(u32, u32)> {
        let tokens = counts[QUOTE as usize];
        if n == 0 || tokens == 0 {
            return None;
        }
        let mut total = 0;
        let mut next = counts.map(|width| {
            let base = total;
            total += width;
            base
        });
        if total != n {
            return None;
        }
        let quote = next[QUOTE as usize];
        let pages = n.div_ceil(64) as usize;
        self.starts.clear();
        self.starts.resize(pages, 0);
        self.page_first.resize(pages, 0);
        self.runs.clear();
        let mut row=0u32;
        let mut previous=None;
        for &rec in caps {
            let byte=(rec>>8) as u8;
            let len=u32::from(rec&255);
            let end=row+len;
            if len==0 || len>127 || end>n {return None;}
            if previous==Some(byte) {
                // A cap boundary inside the same run extends only the exclusive end.
                self.runs.last_mut().unwrap().position_byte += len << 8;
            } else {
                let delta=next[byte as usize].wrapping_sub(row);
                self.runs.push(Run::new(end,byte,delta));
                self.starts[(row>>6) as usize]|=1u64 << (row&63);
                previous=Some(byte);
            }
            next[byte as usize]+=len;
            row=end;
        }
        if row!=n {return None;}
        let mut preceding = 0u32;
        for (mask, base) in self.starts.iter_mut().zip(&mut self.page_first) {
            *base = preceding + (*mask as u32 & 1) - 1;
            preceding += mask.count_ones();
            *mask &= !1;
        }
        Some((quote, tokens))
    }

    fn transfer_mark(&self, p: u32, quote: u32) -> Option<(u32, u32)> {
        let (mut x, mut skip) = (p, 0);
        loop {
            let r = self.runs[self.locate(x)];
            x = r.map(x);
            if r.byte() == QUOTE {
                return Some((x - quote, skip));
            }
            skip += 1;
            if x == p {
                return None;
            }
        }
    }

    fn build(&mut self, quote: u32, tokens: u32) {
        #[cfg(any(target_arch="x86", target_arch="x86_64"))]
        {
            if MODE == 3 && std::is_x86_feature_detected!("popcnt") && std::is_x86_feature_detected!("bmi2") {
                // SAFETY: both features required by this routine were detected on this CPU.
                return unsafe { self.build_bmi(quote, tokens) };
            }
            if MODE != 0 && MODE != 4 && std::is_x86_feature_detected!("popcnt") {
                // SAFETY: the hardware population count feature has been detected.
                return unsafe { self.build_pop(quote, tokens) };
            }
        }
        self.build_body(quote, tokens)
    }

    fn prepare_counted(&mut self, caps: &[u16], n: u32, counts: &[u32;256]) -> Option<(u32,u32)> {
        #[cfg(any(target_arch="x86", target_arch="x86_64"))]
        if MODE >= 2 && std::is_x86_feature_detected!("popcnt") {
            // SAFETY: only the detected hardware population count feature is enabled.
            return unsafe { self.prepare_pop(caps,n,counts) };
        }
        self.prepare_body(caps,n,counts)
    }

    #[cfg(any(target_arch="x86", target_arch="x86_64"))]
    #[target_feature(enable="popcnt")]
    unsafe fn prepare_pop(&mut self, caps: &[u16], n: u32, counts: &[u32;256]) -> Option<(u32,u32)> {
        self.prepare_body(caps,n,counts)
    }

    #[cfg(any(target_arch="x86", target_arch="x86_64"))]
    #[target_feature(enable="popcnt")]
    unsafe fn build_pop(&mut self, quote:u32, tokens:u32) {
        self.build_body(quote,tokens)
    }

    #[cfg(any(target_arch="x86", target_arch="x86_64"))]
    #[target_feature(enable="popcnt,bmi2")]
    unsafe fn build_bmi(&mut self, quote:u32, tokens:u32) {
        self.build_body(quote,tokens)
    }

    #[inline(always)]
    fn build_body(&mut self, quote: u32, tokens: u32) {
        self.nodes.clear();
        self.selectors.clear();
        self.edges.clear();
        self.words.clear();
        self.dict.clear();
        let mut root = Node::new(quote, NONE, QUOTE);
        root.hi += tokens;
        self.nodes.push(root);
        let mut epoch = [NONE; 256];
        let mut active = [0usize; 256];
        #[cfg(slot_counts)] { self.wave_counts=[[0;9];9]; }
        const LANES: usize = 8;
        const ALL: u32 = (1 << LANES) - 1;
        let mut paths: [Vec<u8>; LANES] = std::mem::take(&mut self.lane_paths);
        let mut job = [0usize; LANES];
        let mut lo = [0u32; LANES];
        let mut width = [0u32; LANES];
        let mut index = [0usize; LANES];
        let mut live = 0;
        // Every allocated child has a fully published parent. Allocation order
        // remains a topological ordering even when earlier jobs retire later.
        #[cfg(test)]
        {
            self.histogram = [0; 9];
        }
        let mut next = 0;
        while live != 0 || next < self.nodes.len() {
            let mut empty = ALL ^ live;
            while empty != 0 && next < self.nodes.len() {
                let lane = empty.trailing_zeros() as usize;
                empty &= empty - 1;
                live |= 1 << lane;
                job[lane] = next;
                let n = self.nodes[next];
                next += 1;
                lo[lane] = n.lo;
                width[lane] = n.hi - n.lo;
                paths[lane].clear();
                paths[lane].push(n.first);
            }
            #[cfg(test)]
            {
                self.histogram[live.count_ones() as usize] += 1;
            }
            if live == ALL {
                for lane in 0..LANES {
                    index[lane] = self.locate(lo[lane]);
                }
            } else {
                let mut mask = live;
                while mask != 0 {
                    let lane = mask.trailing_zeros() as usize;
                    mask &= mask - 1;
                    index[lane] = self.locate(lo[lane]);
                }
            }
            let mut done=0;
            if live == ALL {
                if SCHEDULE==1 {
                    for lane in 0..LANES {
                        let r=self.runs[index[lane]];
                        if r.retires(lo[lane]+width[lane]) {
                            done |= 1<<lane;
                        } else {
                            paths[lane].push(r.byte());
                            lo[lane]=r.map(lo[lane]);
                        }
                    }
                } else {
                    let replies:[Run;LANES]=std::array::from_fn(|lane| self.runs[index[lane]]);
                    if SCHEDULE==2 {
                        for lane in 0..LANES {
                            let r=replies[lane];
                            if r.retires(lo[lane]+width[lane]) {
                                done |= 1<<lane;
                            } else {
                                paths[lane].push(r.byte());
                                lo[lane]=r.map(lo[lane]);
                            }
                        }
                    } else {
                        for lane in 0..LANES {
                            done |= u32::from(replies[lane].retires(lo[lane]+width[lane]))<<lane;
                        }
                        if done==0 {
                            for lane in 0..LANES {
                                let r=replies[lane];
                                paths[lane].push(r.byte());
                                lo[lane]=r.map(lo[lane]);
                            }
                        } else {
                            let mut advancing=ALL^done;
                            while advancing!=0 {
                                let lane=advancing.trailing_zeros() as usize;
                                advancing&=advancing-1;
                                let r=replies[lane];
                                paths[lane].push(r.byte());
                                lo[lane]=r.map(lo[lane]);
                            }
                        }
                    }
                }
            } else {
                let mut mask=live;
                while mask!=0 {
                    let lane=mask.trailing_zeros() as usize;
                    mask&=mask-1;
                    let r=self.runs[index[lane]];
                    if r.retires(lo[lane]+width[lane]) {
                        done |= 1<<lane;
                    } else {
                        paths[lane].push(r.byte());
                        lo[lane]=r.map(lo[lane]);
                    }
                }
            }
            #[cfg(slot_counts)]
            {
                self.wave_counts[live.count_ones() as usize][done.count_ones() as usize]+=1;
            }
            live ^= done;
            while done != 0 {
                let lane = done.trailing_zeros() as usize;
                done &= done - 1;
                let i = job[lane];
                let lo = lo[lane];
                let hi = lo + width[lane];
                let mut k = index[lane];
                self.nodes[i].edge = self.edges.len() as u32;
                self.edges.extend_from_slice(&paths[lane]);
                self.nodes[i].end = self.edges.len() as u32;
                self.nodes[i].sel = self.selectors.len() as u32;
                let begin = self.selectors.len();
                let mut a=lo;
                while a<hi {
                    let r=self.runs[k];
                    let end=hi.min(r.position());
                    let width=end-a;
                    let mapped = r.map(a);
                    let c = r.byte() as usize;
                    let id = if r.byte() == QUOTE {
                        if self.nodes[i].word == NONE {
                            self.nodes[i].base = mapped - quote;
                            self.nodes[i].word = self.words.len() as u32;
                            let pos = self.dict.len() as u32;
                            let mut j = i as u32;
                            while j != NONE {
                                let n = self.nodes[j as usize];
                                self.dict.extend(
                                    self.edges[n.edge as usize..n.end as usize].iter().rev(),
                                );
                                j = n.parent;
                            }
                            self.words.push(Piece {
                                id: pos,
                                width: self.dict.len() as u32 - pos,
                            });
                        }
                        NONE
                    } else {
                        if epoch[c] != i as u32 {
                            epoch[c] = i as u32;
                            active[c] = self.nodes.len();
                            self.nodes.push(Node::new(mapped, i as u32, r.byte()));
                        }
                        let target = active[c];
                        debug_assert_eq!(self.nodes[target].hi, mapped);
                        self.nodes[target].hi += width;
                        target as u32
                    };
                    if self.selectors.len() > begin && self.selectors.last().unwrap().id == id {
                        self.selectors.last_mut().unwrap().width += width;
                    } else {
                        self.selectors.push(Piece { id, width });
                    }
                    a=end;
                    k += 1;
                }
                self.nodes[i].sels = self.selectors.len() as u32 - self.nodes[i].sel;
            }
        }
        self.lane_paths = paths;
    }

    #[inline]
    fn emit(merged: &mut Vec<Piece>, start: usize, id: u32, width: u32) {
        if merged.len() > start && merged.last().unwrap().id == id {
            merged.last_mut().unwrap().width += width;
        } else {
            merged.push(Piece { id, width });
        }
    }

    fn inverse_merge(&mut self) {
        self.merged.clear();
        for i in (0..self.nodes.len()).rev() {
            let node = self.nodes[i];
            let start = self.merged.len();
            self.nodes[i].out = start as u32;
            self.nodes[i].cursor = start as u32;
            for k in node.sel..node.sel + node.sels {
                let Piece { id, mut width } = self.selectors[k as usize];
                if id == NONE {
                    Self::emit(&mut self.merged, start, i as u32, width);
                    #[cfg(test)]
                    {
                        self.stats[1] += 1;
                    }
                } else {
                    let child = &mut self.nodes[id as usize];
                    while width > 0 {
                        let piece = self.merged[child.cursor as usize];
                        let count = width.min(piece.width - child.used);
                        Self::emit(&mut self.merged, start, piece.id, count);
                        #[cfg(test)]
                        {
                            self.stats[1] += 1;
                        }
                        width -= count;
                        child.used += count;
                        if child.used == piece.width {
                            child.cursor += 1;
                            child.used = 0;
                        }
                    }
                }
            }
        }
    }

    fn expand(
        &mut self,
        mut row: u32,
        mut skip: u32,
        tokens: u32,
        n: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        let bits = 32 - (tokens - 1).leading_zeros();
        if self.words.len() - 1 > (u32::MAX >> bits) as usize {
            return false;
        }
        let mask = (1u32 << bits) - 1;
        self.links.resize(tokens as usize, 0);
        let mut input = 0;
        for p in &self.merged[self.nodes[0].out as usize..] {
            let node = &mut self.nodes[p.id as usize];
            for j in node.base..node.base + p.width {
                self.links[j as usize] = (node.word << bits) | input;
                input += 1;
            }
            node.base += p.width;
        }
        debug_assert_eq!(input, tokens);
        out.clear();
        out.reserve(n as usize);
        while out.len() < n as usize {
            let link = self.links[row as usize];
            let p = self.words[(link >> bits) as usize];
            let count = (p.width - skip).min(n - out.len() as u32);
            let a = (p.id + skip) as usize;
            out.extend_from_slice(&self.dict[a..a + count as usize]);
            #[cfg(test)]
            {
                self.stats[2] += 1;
            }
            row = link & mask;
            skip = 0;
        }
        true
    }

    #[cfg(test)]
    fn attempt<const FORCE: bool>(
        &mut self,
        caps: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        self.attempt_with::<FORCE>(caps, n, p, out, None)
    }
    fn attempt_with<const FORCE: bool>(
        &mut self,
        caps: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        counts: Option<&[u32; 256]>,
    ) -> bool {
        if n == 0 || n > 900000 || p as usize >= n || caps.len() > n {
            return false;
        }
        if !FORCE && (caps.len() * 20 >= 3 * n || caps.len() * 64 < n) {
            return false;
        }
        #[cfg(test)]
        {
            self.stats = [0; 8];
        }
        let section = match counts {
            Some(freq) => self.prepare_counted(caps, n as u32, freq),
            None => self.prepare(caps, n as u32),
        };
        let Some((quote, tokens)) = section else {
            return false;
        };
        let Some((row, skip)) = self.transfer_mark(p, quote) else {
            return false;
        };
        self.build(quote, tokens);
        self.inverse_merge();
        #[cfg(test)]
        {
            self.stats[3] = skip as u64;
            self.stats[4] = tokens as u64;
        }
        let root = &self.merged[self.nodes[0].out as usize..];
        self.second.invert(
            root,
            &self.nodes,
            &self.words,
            &self.dict,
            row,
            skip,
            tokens,
            n as u32,
            out,
        ) || self.expand(row, skip, tokens, n as u32, out)
    }
}

impl<const MODE:u8,const SCHEDULE:u8> InverseBwt for Dispatch<MODE,SCHEDULE> {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(ll, counts, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(ll, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        self.attempt_with::<false>(r.records(), n, p, out, Some(r.byte_counts()))
            || self.fallback.invert_runs(r, n, p, out, k)
    }
}


#[cfg(test)]
mod checks {
use super::*;
fn verify_marks<const SCHEDULE:u8>() {
    let mut c = Dispatch::<2,SCHEDULE>::default();
    let mut count = 0;
    let mut output = Vec::new();
    for (alphabet, limit) in [(2usize, 9), (3, 6)] {
        for n in 1..=limit {
            for mut seed in 0..alphabet.pow(n as u32) {
                let ll: Vec<_> = (0..n)
                    .map(|_| {
                        let b = (seed % alphabet) as u8 + QUOTE - 1;
                        seed /= alphabet;
                        b
                    })
                    .collect();
                let mut psi: Vec<_> = (0..n).collect();
                psi.sort_by_key(|&i| ll[i]);
                let caps: Vec<_> = ll.iter().map(|&b| (b as u16) << 8 | 1).collect();
                for p in 0..n {
                    let mut x = p;
                    let want: Vec<_> = (0..n)
                        .map(|_| {
                            x = psi[x];
                            ll[x]
                        })
                        .collect();
                    let took = c.attempt::<true>(&caps, n, p as u32, &mut output);
                    assert_eq!(took, want.contains(&QUOTE));
                    if took {
                        assert_eq!(output, want);
                        count += 1;
                    }
                }
            }
        }
    }
    println!("exact section outputs for {count} covered small marks");
}

#[test]
fn arbitrary_marks() { verify_marks::<1>(); verify_marks::<2>(); verify_marks::<3>(); }

#[test]
fn packed_lf_and_boundaries() {
    assert_eq!(size_of::<Run>(),8);
    assert_eq!(align_of::<Run>(),8);
    let mut c=Candidate::default();
    for n in [1usize,31,64,65,358,900000] {
        for widths in [[1usize,17,33,67],[900000,1,1,1],[8189,127,128,25000],[1,1,1,1]] {
            let (mut left,mut i)=(n,0);
            let (mut caps,mut counts)=(Vec::new(),[0u32;256]);
            while left>0 {
                let b=[QUOTE,5,0,255][i%4];
                let len=widths[i%4].min(left);
                counts[b as usize]+=len as u32;
                i+=1;
                left-=len;
                let mut rem=len;
                while rem>0 {
                    let size=rem.min(127);
                    caps.push(u16::from(b)<<8|size as u16);
                    rem-=size;
                }
            }
            let mut cumulative=0;
            let mut lf=counts.map(|v| {let start=cumulative;cumulative+=v;start});
            c.prepare_counted(&caps,n as u32,&counts).unwrap();
            let mut x=0u32;
            for record in caps {
                let b=(record>>8) as u8;
                for _ in 0..record&255 {
                    let k=c.locate(x);
                    let r=c.runs[k];
                    assert_eq!(r.byte(),b);
                    assert_eq!(r.map(x),lf[b as usize]);
                    assert!((k==0 || x>=c.runs[k-1].position()) && x<r.position());
                    lf[b as usize]+=1;
                    x+=1;
                }
            }
            assert_eq!(x,n as u32);
        }
    }
}

}

#[cfg(slot_counts)]
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    pub fn report(&self,block:u64) {
        for active in 1..=8 {
            for retired in 0..=active {
                println!("wave,{block},{active},{retired},{}",self.wave_counts[active][retired]);
            }
        }
    }
}
