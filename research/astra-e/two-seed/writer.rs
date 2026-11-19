use crate::{original, Table};
use std::mem::MaybeUninit;

#[derive(Default, Debug, Clone, Copy)]
pub struct Work {
    pub paired: usize,
    pub carried: usize,
    pub moved: usize,
    pub rejected: usize,
    pub boundary: bool,
}

#[inline]
fn place(code: u32, t: &Table, storage: &mut [MaybeUninit<u8>], pos: usize, end: usize) -> Option<usize> {
    if code < 256 {
        if pos == end {return None;}
        storage[pos].write(code as u8);
        return Some(pos + 1);
    }
    let w = (code - 256) as usize;
    let word = &t.spelling[t.offsets[w] as usize..t.offsets[w + 1] as usize];
    if word.len() > end-pos {return None;}
    let dest = &mut storage[pos..pos + word.len()];
    // The spelling slice and the output belong to separate allocations.
    unsafe {std::ptr::copy_nonoverlapping(word.as_ptr(),dest.as_mut_ptr().cast::<u8>(),word.len());}
    Some(pos + word.len())
}

#[inline]
fn write_to(t: &Table, store: &mut [MaybeUninit<u8>], mut pos: usize, end: usize,
            mut row: u32, a: u32, b: u32) -> (usize,u32) {
    let mask = (1 << t.bits) - 1;
    let mut edge = t.edges[row as usize];
    loop {
        let code = edge >> t.bits;
        row = edge & mask;
        edge = t.edges[row as usize];
        pos = place(code,t,store,pos,end).expect("cycle weight exceeds supplied extent");
        if row == a || row == b {return (pos,row);}
    }
}

#[inline]
fn finish(t: &Table, out: &mut Vec<u8>, len: usize) {
    assert!(len > 0 && len <= t.n);
    // The owner writes a contiguous initialized prefix; no uninitialized gap is exposed.
    unsafe {out.set_len(len);}
    while out.len() < t.n {
        let take = out.len().min(t.n-out.len());
        out.extend_from_within(..take);
    }
}

pub fn single(t: &Table, out: &mut Vec<u8>) {
    if !t.extra.is_empty() {return original::Kernel::stock(t,out);}
    out.clear();
    out.reserve(t.n);
    let (length,_) = write_to(t,&mut out.spare_capacity_mut()[..t.n],0,t.n,t.mark,t.mark,t.mark);
    finish(t,out,length);
}

// METERS compiles away counters in the paid binary. The split parameter is also used
// by the independent exhaustive bounds checks, including zero-room starts.
pub fn split_at<const METERS: bool, const FREEZE: bool>(t: &Table, out: &mut Vec<u8>, seed: u32, cut: usize) -> Work {
    let mut work = Work::default();
    if seed == t.mark || !t.extra.is_empty() {
        original::Kernel::stock(t,out);
        return work;
    }
    out.clear();
    out.reserve(t.n);
    let storage = &mut out.spare_capacity_mut()[..t.n];
    let mask = (1 << t.bits) - 1;
    let mut a=t.mark;
    let mut b=seed;
    let mut ea=t.edges[a as usize];
    let mut eb=t.edges[b as usize];
    let mut left=0;
    let mut right=cut;
    loop {
        let ca = ea >> t.bits;
        let cb = eb >> t.bits;
        let na = ea & mask;
        let nb = eb & mask;
        // Expose independent successor addresses before expanding either current label.
        ea=t.edges[na as usize];
        eb=t.edges[nb as usize];
        let Some(next) = place(ca,t,storage,left,cut) else {
            if METERS {work.boundary=true;}
            // The complete A arc exceeds cut, so the full B arc fits after cut.
            // If B proves foreign, its own cycle also fits in that reservation.
            if FREEZE {(right,b)=write_to(t,storage,right,t.n,b,seed,t.mark);}
            break;
        };
        left=next;
        a=na;
        let Some(next) = place(cb,t,storage,right,t.n) else {
            if METERS {work.boundary=true;}
            // B exceeds its reservation. The complete A arc fits below cut.
            if FREEZE && a!=t.mark && a!=seed {
                (left,a)=write_to(t,storage,left,cut,a,seed,t.mark);
            }
            break;
        };
        right=next;
        b=nb;
        if METERS {work.paired+=1;}
        if a==t.mark || a==seed || b==seed || b==t.mark {break;}
    }
    let kept = right-cut;
    if METERS {work.carried=kept;}
    let mut source=cut;
    // If either lane returns to its own seed, the cycles are disjoint.
    let unrelated=(left>0 && a==t.mark) || (kept>0 && b==seed);
    if unrelated {
        if METERS {work.rejected=kept;}
        if left==0 || a!=t.mark {
            (left,_) = write_to(t,storage,left,t.n,a,t.mark,t.mark);
        }
    } else {
        if a!=seed {
            source=t.n-kept;
            if source!=cut && kept>0 {
                storage.copy_within(cut..right,source);
                if METERS {work.moved+=kept;}
            }
            (left,a)=write_to(t,storage,left,source,a,seed,t.mark);
        }
        if a==seed {
            if source!=left && kept>0 {
                storage.copy_within(source..source+kept,left);
                if METERS {work.moved+=kept;}
            }
            left+=kept;
            if b!=t.mark {
                (left,_) = write_to(t,storage,left,t.n,b,t.mark,t.mark);
            }
        } else if METERS {work.rejected=kept;}
    }
    finish(t,out,left);
    work
}

pub fn split<const METERS: bool, const FREEZE: bool>(t: &Table, out: &mut Vec<u8>) -> Work {
    let m=t.edges.len();
    let seed=(t.mark as usize+m/2)%m;
    split_at::<METERS,FREEZE>(t,out,seed as u32,t.n/2)
}
