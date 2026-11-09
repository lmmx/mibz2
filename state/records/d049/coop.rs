    fn named(k: usize, prefix: &str) -> std::io::Result<Pool> {
        let k = k.max(1);
        let shared = Arc::new(Shared {
            epoch: AtomicUsize::new(0),
            done: AtomicUsize::new(0),
            task: Mutex::new(None),
            panic: Mutex::new(None),
            caller: Mutex::new(None),
        });
        let mut pool = Pool {
            k: 1,
            shared,
            helpers: Vec::with_capacity(k - 1),
        };
        for idx in 1..k {
            let shared = Arc::clone(&pool.shared);
            let h = thread::Builder::new()
                .name(format!("{prefix}{idx}"))
                .spawn(move || helper(&shared, idx))?;
            // (on an error above, `pool` drops with the helpers spawned so far, its k
            // counting only them)
            pool.helpers.push(h);
            pool.k = idx + 1;
        }
        Ok(pool)
    }

    /// The pool's threads, the caller included
    pub(crate) fn threads(&self) -> usize {
        self.k
    }

    /// `f(0)` on the caller and `f(j)` on helper j, returning when all k have ended. A
    /// panic in any of them is resumed here once all have ended: the caller's own if it

    /// Range j's scatter (j below the shares), on any thread; a j already claimed, or out
    /// of range, writes nothing
    pub(crate) fn share(&self, j: usize) {
        if j >= self.shares {
            return;
        }
        let bit = 1u64 << j;
        if self.claimed.fetch_or(bit, Ordering::Relaxed) & bit != 0 {
            return;
