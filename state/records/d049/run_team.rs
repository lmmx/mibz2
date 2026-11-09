/// [q, end): w starts at q and only rises; a vector store covers [w, w + 8) and is issued
/// only while w + 8 <= end; a scalar one writes w < stop <= end. So the tail at an
/// ownership edge is exact (scalar), not an 8-word overstore: a share of one row (or any
/// range shorter than 8) gets no vector store at all, and no store reaches the table's
/// n-th row (no n + 8 slack is relied on). The callers pass as `end` the end of the rows
/// the thread owns: `fill_rows_at` its share's q1; `fill_direct_at` the end of byte c's
/// F range in its share (`merge_rows`' hist[2][c]), checked to contain the run before
/// any store. Those ranges are disjoint across threads by the prefix's construction, so
/// no store from one share's fill lands in another's rows. Inside its own range, a vector
/// store's words past `stop` are rows the same thread writes again later, because it
/// fills that range in rising order to its end (checked: a fill that stops short
/// returns false, and the table is then never exposed). Tests: `fills_write_their_own_
/// rows_only` (each share alone into a table of sentinels: exactly its rows changed, its
/// distinct words its rows), `miri_parallel_footprint` (every share at once, one-row and
/// one-record shares among them, under Miri's race detector with AVX2).
///
/// SAFETY: q <= stop <= end, and rows [q, end) of `t` are valid for writes and written by
/// no other thread meanwhile (with `AVX`, AVX2 is available)
#[inline(always)]
unsafe fn cell<const AVX: bool>(t: *mut u32, q: u32, stop: u32, end: u32, base: u32) -> u64 {
    let mut w = q;
    let mut stored = 0u64;
    #[cfg(target_arch = "x86_64")]
    if AVX {
        use std::arch::x86_64::*;
        let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        let step = _mm256_set1_epi32(2048);
        let mut v = _mm256_add_epi32(_mm256_set1_epi32(base as i32), lanes);
        while w < stop && w + 8 <= end {
            // SAFETY: rows [w, w + 8) are inside [q, end)
            unsafe { _mm256_storeu_si256(t.add(w as usize) as *mut __m256i, v) };
            v = _mm256_add_epi32(v, step);
            w += 8;
            stored += 8;
        }
    }
    while w < stop {
        // SAFETY: w is inside [q, stop)
        unsafe { t.add(w as usize).write(base + ((w - q) << 8)) };
        w += 1;
        stored += 1;
    }
    stored
}

/// Rows [q0, q1) of the table at `t` from L-order records and F-order ones whose tilings
    #[allow(clippy::too_many_arguments)]
    fn new(
        recs: &'a [u32],
        counts: &'a [u32; 256],
        n: usize,
        presorted: bool,
        fo_hs: &'a mut Vec<[u32; 2]>,
        hist: &'a mut Vec<[[u32; 256]; 3]>,
        shares: usize,
        tt: &'a mut TVector,
        direct: bool,
        t0: Instant,
        inject: u8,
    ) -> RunFill<'a> {
        let r = recs.len().saturating_sub(1);
        // the leader's sorted F order (read only), or the helpers' to scatter into
        let sorted = presorted && fo_hs.len() == r + 1;
        if !presorted {
            fo_hs.clear();
            if !direct {
                fo_hs.reserve(r + 1);
            }
        }
        let fo = fo_hs.as_mut_ptr();
        hist.clear();
        hist.resize(shares, [[0; 256]; 3]);
        tt.0.clear();
        tt.0.reserve(n);
        let mut cuts = [0; MAX_SHARES + 1];
        if direct {
            row_cuts(recs, n, shares, RECORD_WEIGHT, &mut cuts);
        }
        RunFill {
            recs,
            counts,
            n,
            r,
            shares,
            fo,
            hist: hist.as_mut_ptr(),
            tt_p: tt.0.as_mut_ptr(),
            tt,
            _bufs: std::marker::PhantomData,
            counted: Gate::new(),
            scattered_gate: Gate::new(),
            unrisen: AtomicBool::new(false),
            sorted: AtomicBool::new(sorted),
            scattered: AtomicU64::new(0),
            claimed: AtomicU64::new(0),
            filled: AtomicU64::new(0),
            direct,
            cuts,
            t0,
            census: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
            inject,
        }
    }

    /// Share j's count (the helpers' sort)
    fn count(&self, j: usize) {
        self.injected(INJECT_COUNT, j);
        let (lo, hi) = self.records_of(j);
        // SAFETY: entry j is this share's alone until the counted gate
        let h = unsafe { &mut *self.hist.add(j) };
        let ok = if self.direct {
            count_rows(self.recs, lo, hi, h)
        } else {
            count_share(self.recs, lo, hi, h)
        };
        if !ok {
            self.unrisen.store(true, Ordering::Relaxed);
        }
    }

    /// The prefix, on the last helper to count (after the counted gate's acquire)
    fn merge(&self) {
        self.injected(INJECT_MERGE, 0);
        // SAFETY: every share has counted (the gate) and none touches the entries until
        // the gate opens
        let hists = unsafe { std::slice::from_raw_parts_mut(self.hist, self.shares) };
        if self.unrisen.load(Ordering::Relaxed) {
            return;
        }
        if self.direct {
            if merge_rows(self.recs, self.counts, self.n, hists) {
                self.sorted.store(true, Ordering::Relaxed);
            }
        } else if merge_shares(self.recs, self.counts, self.n, hists) {
            // SAFETY: r + 1 slots were reserved; slot r is no share's (theirs tile [0, r))
            unsafe { self.fo.add(self.r).write([self.n as u32, 0]) };
            self.sorted.store(true, Ordering::Relaxed);
        }
    }

    /// Share j's scatter, if the merge passed
    let helpers = pool.threads().saturating_sub(1).max(1);
    let shares = helpers.min(MAX_SHARES);
    let t0 = Instant::now();
    add(&PH.runs, 1);
        helper(0);
    } else {
        pool.run(&|j| {
            if j == 0 {
                leader();
                relay.join();
            } else {
                helper(j - 1);
            }
        });
    }
            Err(p) => {
                let msg = panic_message(p);
                let held = [Some(s), l.next.take()];
                for b in held.into_iter().flatten() {
                    let kind = ErrorKind::Internal(msg.clone());
                    publish(shared, b.place, b.bit, Err(kind), b.symbols.syms);
                }
                if let Some(b) = l.inverted.take() {
                    let kind = ErrorKind::Internal(msg.clone());
                    publish(shared, b.place, b.bit, Err(kind), b.symbols.syms);
                }
                t = Team::default();
                continue;
            }
        }
        staged = l.next.take();
    }
impl StreamDecoder {
    /// Research, opt-in (`run-fed`), not the default: `teams` pipelined teams of `k`
    /// threads (at least 2) with the shipped levers (park and small), their leaders
    /// emitting run records instead of the last column, and the sort where `sort` says
    /// (docs/journal/2026-11-09-run-fed-leader.md). Byte for byte the default decoder's
    /// output and errors.
    #[doc(hidden)]
    pub fn pipelined_run_fed(
        teams: usize,
        k: usize,
        slots: usize,
        kernels: Kernels,
        sort: RunFedSort,
    ) -> StreamDecoder {
        let (teams, team_k) = (teams.max(1), k.max(2));
        let mut dec = match sort {
            RunFedSort::Control => Self::launch(teams, slots, move |s, j| {
                worker::<CTL>(s, j, kernels, team_k)
            }),
            RunFedSort::Leader => Self::launch(teams, slots, move |s, j| {
                worker::<LS>(s, j, kernels, team_k)
            }),
            RunFedSort::Helpers => Self::launch(teams, slots, move |s, j| {
                worker::<HS>(s, j, kernels, team_k)
            }),
            RunFedSort::RegControl => Self::launch(teams, slots, move |s, j| {
                worker::<CTLR>(s, j, kernels, team_k)
            }),
            RunFedSort::Direct => Self::launch(teams, slots, move |s, j| {
                worker::<DIR>(s, j, kernels, team_k)
            }),
        };
        dec.threads = teams * team_k;
        dec.pipelined = true;
        dec
    }
}

#[cfg(test)]
mod tests {
