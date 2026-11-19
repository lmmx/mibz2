// Independent scalar specification. No decoder or BMI2 intrinsics enter the model.
#[derive(Clone, PartialEq)]
struct History {
    window: usize,
    n: usize,
    mask: u64,
    ring: [u8; 64],
    head: usize, // next overwritten ring slot
    tail: [u8; 256],
    right: [u8; 256], // independent right-aligned layout: [popcount(mask)..n)
    tail_head: usize, // oldest segment's front in cyclic storage
    tail_len: usize,
}
#[derive(Copy, Clone, Debug, PartialEq)]
enum Route {
    Hit,
    Miss,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Event {
    Run(u8),
    End,
    Literal(u8, Route),
}

impl History {
    fn reset(n: usize, window: usize) -> Self {
        assert!((1..=256).contains(&n) && (1..=64).contains(&window));
        Self {
            n,
            window,
            mask: 0,
            head: 0,
            ring: [255; 64],
            tail: std::array::from_fn(|i| i as u8),
            right: std::array::from_fn(|i| i as u8),
            tail_head: 0,
            tail_len: n,
        }
    }
    fn slot(&self, age: usize) -> usize {
        (self.head + self.window - 1 - age) % self.window
    }
    fn at_tail(&self, j: usize) -> usize {
        (self.tail_head + j) % self.n
    }
    fn take_tail(&mut self, j: usize) -> u8 {
        assert!(j < self.tail_len);
        let chosen = self.tail[self.at_tail(j)];
        if j < self.tail_len - 1 - j {
            for i in (1..=j).rev() {
                self.tail[self.at_tail(i)] = self.tail[self.at_tail(i - 1)];
            }
            self.tail_head = self.at_tail(1);
        } else {
            for i in j..self.tail_len - 1 {
                self.tail[self.at_tail(i)] = self.tail[self.at_tail(i + 1)];
            }
        }
        self.tail_len -= 1;
        chosen
    }
    fn prepend(&mut self, id: u8) {
        assert!(self.tail_len < self.n);
        self.tail_head = (self.tail_head + self.n - 1) % self.n;
        self.tail[self.tail_head] = id;
        self.tail_len += 1;
    }
    fn step(&mut self, rank: usize) -> (u8, Route) {
        assert!(rank < self.n);
        let active = self.mask.count_ones() as usize;
        let mut first_old = active;
        let (id, route) = if rank < active {
            let mut suffix = self.mask;
            for _ in 0..rank {
                suffix &= suffix - 1;
            }
            let age = suffix.trailing_zeros() as usize;
            self.mask &= !(1u64 << age);
            (self.ring[self.slot(age)], Route::Hit)
        } else {
            let ordinal = self.take_tail(rank - active);
            assert_eq!(self.right[rank], ordinal);
            self.right.copy_within(active..rank, active + 1);
            first_old += 1;
            (ordinal, Route::Miss)
        };
        if self.mask & (1u64 << (self.window - 1)) != 0 {
            self.prepend(self.ring[self.head]);
            first_old -= 1;
            self.right[first_old] = self.ring[self.head];
        }
        self.mask = ((self.mask << 1) | 1) & (u64::MAX >> (64 - self.window));
        assert_eq!(first_old, self.mask.count_ones() as usize);
        assert_eq!(self.tail_len, self.n - first_old);
        for j in 0..self.tail_len {
            assert_eq!(self.right[first_old + j], self.tail[self.at_tail(j)]);
        }
        self.ring[self.head] = id;
        self.head = (self.head + 1) % self.window;
        (id, route)
    }
    fn observe(&mut self, symbol: u16, eob: u16) -> Event {
        assert_eq!(self.n, 256);
        if symbol <= 1 {
            return Event::Run(if self.mask == 0 {
                self.right[0]
            } else {
                self.ring[self.slot(0)]
            });
        }
        if symbol == eob {
            return Event::End;
        }
        let (id, route) = self.step(usize::from(symbol - 1).min(255));
        Event::Literal(id, route)
    }
    fn list(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.n);
        for age in 0..self.window {
            if self.mask >> age & 1 != 0 {
                result.push(self.ring[self.slot(age)]);
            }
        }
        for j in 0..self.tail_len {
            result.push(self.tail[self.at_tail(j)]);
        }
        result
    }
    // Newest-to-oldest literal identities. Inactive slots are arbitrary.
    fn with_layout(
        n: usize,
        window: usize,
        head: usize,
        tail_head: usize,
        recent: &[u8],
        older: &[u8],
    ) -> Self {
        let mut state = Self::reset(n, window);
        state.head = head;
        state.tail_head = tail_head;
        state.tail_len = older.len();
        state.right[n - older.len()..n].copy_from_slice(older);
        let mut seen = [false; 256];
        for (age, &id) in recent.iter().enumerate() {
            let slot = state.slot(age);
            state.ring[slot] = id;
            if !seen[id as usize] {
                state.mask |= 1 << age;
                seen[id as usize] = true;
            }
        }
        for (i, &id) in older.iter().enumerate() {
            state.tail[state.at_tail(i)] = id;
        }
        state
    }
}

fn permutations(tail: &mut [u8], i: usize, f: &mut impl FnMut(&[u8])) {
    if i == tail.len() {
        f(tail);
        return;
    }
    for j in i..tail.len() {
        tail.swap(i, j);
        permutations(tail, i + 1, f);
        tail.swap(i, j);
    }
}

fn validate_small(n: usize, w: usize) -> usize {
    let mut checks = 0;
    for length in 0..=w {
        for encoded in 0..n.pow(length as u32) {
            let mut z = encoded;
            let recent: Vec<u8> = (0..length)
                .map(|_| {
                    let id = z % n;
                    z /= n;
                    id as u8
                })
                .collect();
            let mut front = Vec::new();
            for &id in &recent {
                if !front.contains(&id) {
                    front.push(id);
                }
            }
            let mut absent: Vec<u8> = (0..n as u8).filter(|id| !front.contains(id)).collect();
            permutations(&mut absent, 0, &mut |older| {
                let ordered = [&front[..], older].concat();
                for h in 0..w {
                    for t in 0..n {
                        let state = History::with_layout(n, w, h, t, &recent, older);
                        assert_eq!(state.list(), ordered);
                        for rank in 0..n {
                            let mut next = state.clone();
                            let (chosen, route) = next.step(rank);
                            assert_eq!(
                                route,
                                if rank < front.len() {
                                    Route::Hit
                                } else {
                                    Route::Miss
                                }
                            );
                            let mut expected = ordered.clone();
                            expected.remove(rank);
                            expected.insert(0, ordered[rank]);
                            assert_eq!(chosen, ordered[rank]);
                            assert_eq!(next.list(), expected);
                            let past: Vec<_> = std::iter::once(chosen)
                                .chain(recent.iter().copied())
                                .take(w)
                                .collect();
                            let expected_mask = past.iter().enumerate().fold(0u64, |m, (a, id)| {
                                m | (u64::from(!past[..a].contains(id)) << a)
                            });
                            assert_eq!(next.mask, expected_mask);
                            checks += 1;
                        }
                    }
                }
            });
        }
    }
    checks
}

fn full_width() -> usize {
    let mut state = History::reset(256, 64);
    let mut list: Vec<u8> = (0..=255).collect();
    let mut checkpoints = 0;
    for n in 0..2304usize {
        // Fill with 64 distinct prior ordinals, including the oldest bit. Then exercise
        // the full/partial windows, high-rank misses, clamp, and both tail directions.
        let symbol = if n < 64 {
            65
        } else if n % 511 == 0 {
            257
        } else {
            [
                64u16,
                65,
                66,
                129,
                256,
                33,
                65535,
                2,
                0,
                1,
                17,
                (2 + (n * 113) % 255) as u16,
            ][n % 12]
        };
        // 257 is EOB only in a 256-byte alphabet. The direct clamp check below also
        // covers synthetic out-of-alphabet symbols after removing that sentinel.
        if symbol == 257 {
            let before = state.clone();
            assert_eq!(state.observe(symbol, 257), Event::End);
            assert!(state == before);
            state = History::reset(256, 64);
            list = (0..=255).collect();
            assert_eq!(state.list(), list);
            continue;
        }
        if symbol < 2 {
            let before = state.clone();
            assert_eq!(state.observe(symbol, 257), Event::Run(list[0]));
            assert!(state == before);
            continue;
        }
        if n == 64 {
            assert_eq!(state.mask, u64::MAX);
        }
        let rank = usize::from(symbol - 1).min(255);
        let mut trial_ranks: Vec<usize> = if n == 64 || n % 37 == 0 {
            (0..256).collect()
        } else {
            vec![0, 1, 63, 64, 65, 127, 128, 254, 255]
        };
        trial_ranks.push(rank);
        for test in trial_ranks {
            let mut trial = state.clone();
            let (id, route) = trial.step(test);
            assert_eq!(id, list[test]);
            if test >= 64 {
                assert_eq!(route, Route::Miss);
            }
            let mut ordered = list.clone();
            ordered.remove(test);
            ordered.insert(0, id);
            assert_eq!(trial.list(), ordered);
            checkpoints += 1;
        }
        let chosen = list.remove(rank);
        list.insert(0, chosen);
        let route = if rank < state.mask.count_ones() as usize {
            Route::Hit
        } else {
            Route::Miss
        };
        assert_eq!(state.observe(symbol, 257), Event::Literal(chosen, route));
        assert_eq!(state.list(), list);
    }
    let mut duplicate = History::reset(3, 2);
    let mapping = [17, 17, 29];
    let a = duplicate.step(1).0;
    let b = duplicate.step(1).0;
    assert_ne!(a, b);
    assert_eq!(mapping[a as usize], mapping[b as usize]);
    assert_eq!(duplicate.mask, 3);
    assert_eq!(duplicate.list(), [0, 1, 2]);
    checkpoints
}

fn main() {
    let mut total = 0;
    for n in 1..=4 {
        let checks = (1..=6).map(|w| validate_small(n, w)).sum::<usize>();
        println!("alphabet={n} windows=1..6 checked={checks}");
        total += checks;
    }
    println!(
        "full_window_checks={} duplicate_label_case=PASS",
        full_width()
    );
    println!("small_exhaustive_checks={total}");
}
