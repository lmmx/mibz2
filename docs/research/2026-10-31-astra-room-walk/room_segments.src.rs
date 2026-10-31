fn room_segments<const L: usize, const PF: u8>(
    tt: &mut TVector,
    orig_ptr: u32,
    s: &mut SegScratch,
    segments: usize,
) {
    let n = tt.0.len();
    let r0 = tt.0[orig_ptr as usize] >> 8;
    random_rows(n, segments - 1, &mut s.starts);
    s.starts.push(r0);
    s.starts.sort_unstable();
    s.starts.dedup();
    let ns = s.starts.len();
    for &r in &s.starts {
        tt.0[r as usize] |= START;
    }
    let cap = 2 * n / ns + 64;
    s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
    for b in &mut s.bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    s.next.clear();
    s.next.resize(ns, usize::MAX);
    let starts = &s.starts;
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let t = tt.0.as_ptr();
    let bufs = &mut s.bufs;
    let next = &mut s.next;

    let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
        let b = &mut bufs[seg];
        let base = b.as_mut_ptr();
        // SAFETY: the starts are rows below n; the buffer has `cap` > 0 spare bytes
        let q = unsafe { *t.add(starts[seg] as usize) } & LINK;
        unsafe { *base = q as u8 };
        Lane {
            row: q >> 8,
            seg,
            p: unsafe { base.add(1) },
            end: unsafe { base.add(b.capacity()) },
        }
    };
    let finish = |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
        let b = &mut bufs[l.seg];
        // SAFETY: p is within b's allocation, every byte below it written
        unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
        next[l.seg] = seg_of(l.row);
    };
    let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
        transition!(GROWS);
        let b = &mut bufs[l.seg];
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        b.reserve(b.capacity());
        l.p = unsafe { b.as_mut_ptr().add(len) };
        l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
    };

    let mut queued = 0;
    let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
        row: 0,
        seg: usize::MAX,
        p: std::ptr::null_mut(),
        end: std::ptr::null_mut(),
    });
    let mut active = 0;
    while active < L && queued < ns {
        lanes[active] = take(queued, bufs);
        if PF != PF_OFF {
            prefetch::<PF>(t, lanes[active].row);
        }
        queued += 1;
        active += 1;
    }
    if active == L {
        'full: loop {
            // The pass's room: what every lane has spare, and no more than a buffer taken
            // mid-pass has after its first byte (`cap` - 1)
            let mut room = cap - 1;
            for l in lanes.iter_mut() {
                if l.p == l.end {
                    grow(l, bufs);
                }
                room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
            }
            debug_assert!(room > 0);
            if lanes
                .iter()
                .any(|l| unsafe { l.end.offset_from(l.p) } as usize == room)
            {
                transition!(PASS_ENDS_FULL);
            }
            let mut base: [*mut u8; L] = std::array::from_fn(|k| lanes[k].p);
            for off in 0..room {
                for k in 0..L {
                    let l = &mut lanes[k];
                    // SAFETY: rows are below n (TVector), with the flag masked off below
                    let q = unsafe { *t.add(l.row as usize) };
                    if q & START != 0 {
                        l.p = base[k].wrapping_add(off);
                        finish(l, bufs, next);
                        if queued == ns {
                            // the lanes before this one wrote at `off`, those after it not yet
                            for (j, lj) in lanes.iter_mut().enumerate() {
                                if j < k {
                                    lj.p = base[j].wrapping_add(off + 1);
                                } else if j > k {
                                    lj.p = base[j].wrapping_add(off);
                                }
                            }
                            transition!(FULL_PHASE_ENDS_MID_PASS);
                            break 'full;
                        }
                        transition!(MID_PASS_TAKES);
                        *l = take(queued, bufs);
                        if PF != PF_OFF {
                            prefetch::<PF>(t, l.row);
                        }
                        queued += 1;
                        // its next byte, at offset off + 1, goes at its p
                        base[k] = l.p.wrapping_sub(off + 1);
                        continue;
                    }
                    l.row = q >> 8;
                    if PF != PF_OFF {
                        prefetch::<PF>(t, l.row);
                    }
                    // SAFETY: base + off is below the lane's end: room is at most its spare
                    // at the pass's start (or, for a buffer taken mid-pass, its capacity
                    // after its first byte)
                    debug_assert!(base[k].wrapping_add(off) < l.end);
                    unsafe { *base[k].wrapping_add(off) = q as u8 };
                }
            }
            for (l, b) in lanes.iter_mut().zip(base) {
                l.p = b.wrapping_add(room);
            }
        }
    }
    let mut live: Vec<Lane> = lanes[..active]
        .iter()
        .copied()
        .filter(|l| next_unset(l, next))
        .collect();
    // A pass can end with a lane's buffer exactly full (its room was all its spare): the
    // tail writes before it tests, so each lane comes into it with a byte to spare
    for l in live.iter_mut() {
        if l.p == l.end {
            transition!(TAIL_ENTERED_FULL);
            grow(l, bufs);
        }
        debug_assert!(l.p < l.end);
    }
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let q = unsafe { *t.add(l.row as usize) };
            if q & START != 0 {
                finish(l, bufs, next);
                live.swap_remove(i);
                continue;
            }
            l.row = q >> 8;
            if PF != PF_OFF {
                prefetch::<PF>(t, l.row);
            }
            unsafe {
                *l.p = q as u8;
                l.p = l.p.add(1);
            }
            if l.p == l.end {
                grow(l, bufs);
            }
            i += 1;
        }
    }
    for &r in starts {
        tt.0[r as usize] &= LINK;
    }
    s.first = seg_of(r0);
    s.n = n;
}
