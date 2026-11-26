//! Research only (`scan-oracle`, `supplied-codes`, `ideal-*`): lock-free `OnceLock` snapshot
//! container for recorded oracle stores (`IDEAL-PARTS-2.md` §3.2).
//!
//! The first [`FrozenSlot::freeze`] publishes the recorded store snapshot into
//! `std::sync::OnceLock<S>`. Every subsequent `Replay` lookup executes zero locks, zero `Arc`
//! refcount operations, and zero `thread_local!` borrows (just a generation check and
//! `OnceLock::get()`). Repeated `set_mode(Replay)` calls on an unchanged store are instant
//! no-ops and preserve the `OnceLock` fast path across all arms and decodes.
//!
//! If [`FrozenSlot::invalidate`] (`clear_store()`) is called after `once` has already been
//! populated (e.g. multi-corpus integration tests switching corpora within one process),
//! subsequent `freeze()` calls publish a generation-tagged `Arc<S>` cached once per worker
//! thread in its `thread_local!` slot — still zero locks per block after each thread's first
//! block.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::thread::LocalKey;

/// Per-thread cache for the multi-corpus fallback snapshot (`(generation, Arc<S>)`).
pub(crate) type TlsCache<S> = RefCell<(u64, Option<Arc<S>>)>;

/// Lock-free read-only snapshot slot backed by `OnceLock<S>` with a multi-corpus TLS fallback.
pub(crate) struct FrozenSlot<S: Send + Sync + 'static> {
    once: OnceLock<S>,
    once_gen: AtomicU64,
    active_gen: AtomicU64,
    store_ver: AtomicU64,
    frozen_ver: AtomicU64,
    fallback: RwLock<Option<(u64, Arc<S>)>>,
    lock_acquires: AtomicU64,
}

impl<S: Send + Sync + 'static> FrozenSlot<S> {
    pub(crate) const fn new() -> Self {
        Self {
            once: OnceLock::new(),
            once_gen: AtomicU64::new(0),
            active_gen: AtomicU64::new(0),
            store_ver: AtomicU64::new(1),
            frozen_ver: AtomicU64::new(0),
            fallback: RwLock::new(None),
            lock_acquires: AtomicU64::new(0),
        }
    }

    /// Mark the mutable recording store modified so the next [`Self::freeze`] captures it.
    #[inline]
    pub(crate) fn mark_dirty(&self) {
        self.store_ver.fetch_add(1, Ordering::Release);
        self.active_gen.store(0, Ordering::Release);
    }

    /// Invalidate the active frozen snapshot (called by `clear_store()`).
    pub(crate) fn invalidate(&self) {
        self.store_ver.fetch_add(1, Ordering::SeqCst);
        self.frozen_ver.store(0, Ordering::SeqCst);
        self.active_gen.store(0, Ordering::SeqCst);
        *self.fallback.write().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// Freeze the current recording store into the lock-free snapshot if not already frozen at
    /// the current version.
    pub(crate) fn freeze(&self, build: impl FnOnce() -> Option<S>) {
        let ver = self.store_ver.load(Ordering::Acquire);
        if self.frozen_ver.load(Ordering::Acquire) == ver
            && self.active_gen.load(Ordering::Acquire) != 0
        {
            return;
        }
        let mut guard = self.fallback.write().unwrap_or_else(|e| e.into_inner());
        let ver = self.store_ver.load(Ordering::Acquire);
        if self.frozen_ver.load(Ordering::Acquire) == ver
            && self.active_gen.load(Ordering::Acquire) != 0
        {
            return;
        }
        let Some(snapshot) = build() else {
            *guard = None;
            self.active_gen.store(0, Ordering::SeqCst);
            return;
        };
        if self.once.get().is_none() {
            match self.once.set(snapshot) {
                Ok(()) => {
                    self.once_gen.store(ver, Ordering::Release);
                    self.frozen_ver.store(ver, Ordering::Release);
                    self.active_gen.store(ver, Ordering::Release);
                    return;
                }
                Err(s) => {
                    *guard = Some((ver, Arc::new(s)));
                }
            }
        } else {
            *guard = Some((ver, Arc::new(snapshot)));
        }
        self.frozen_ver.store(ver, Ordering::Release);
        self.active_gen.store(ver, Ordering::Release);
    }

    /// Whether the pure `OnceLock` fast path is currently active.
    pub(crate) fn is_once_active(&self) -> bool {
        let gen = self.active_gen.load(Ordering::Acquire);
        gen != 0 && gen == self.once_gen.load(Ordering::Relaxed) && self.once.get().is_some()
    }

    /// Fallback `RwLock::read()` acquisitions during `Replay` since the last call (and reset).
    /// Always 0 on the `OnceLock` fast path.
    pub(crate) fn take_lock_acquires(&self) -> u64 {
        self.lock_acquires.swap(0, Ordering::SeqCst)
    }

    /// Borrow the frozen snapshot `&S` lock-free and invoke `f(&S)`.
    #[inline]
    pub(crate) fn with<R>(
        &self,
        tls: &'static LocalKey<TlsCache<S>>,
        f: impl FnOnce(&S) -> R,
    ) -> Option<R> {
        let gen = self.active_gen.load(Ordering::Acquire);
        if gen == 0 {
            return None;
        }
        if gen == self.once_gen.load(Ordering::Relaxed) {
            if let Some(snapshot) = self.once.get() {
                return Some(f(snapshot));
            }
        }
        self.with_fallback(tls, gen, f)
    }

    #[cold]
    fn with_fallback<R>(
        &self,
        tls: &'static LocalKey<TlsCache<S>>,
        gen: u64,
        f: impl FnOnce(&S) -> R,
    ) -> Option<R> {
        let needs_refresh = tls.with(|cell| cell.borrow().0 != gen);
        if needs_refresh {
            self.lock_acquires.fetch_add(1, Ordering::Relaxed);
            let fetched = {
                let g = self.fallback.read().unwrap_or_else(|e| e.into_inner());
                g.as_ref()
                    .and_then(|(fg, arc)| (*fg == gen).then(|| Arc::clone(arc)))
            };
            tls.with(|cell| {
                *cell.borrow_mut() = match fetched {
                    Some(arc) => (gen, Some(arc)),
                    None => (0, None),
                };
            });
        }
        tls.with(|cell| {
            let b = cell.borrow();
            let arc = b.1.as_ref()?;
            Some(f(arc))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_slot_uses_oncelock_then_tls_fallback() {
        type Snap = Box<[Option<u64>]>;
        static SLOT: FrozenSlot<Snap> = FrozenSlot::new();
        thread_local! {
            static TLS: TlsCache<Snap> = const { RefCell::new((0, None)) };
        }
        SLOT.mark_dirty();
        SLOT.freeze(|| Some(vec![Some(10), None, Some(30)].into_boxed_slice()));
        assert!(SLOT.is_once_active());
        assert_eq!(SLOT.with(&TLS, |s| s[0]), Some(Some(10)));
        assert_eq!(SLOT.with(&TLS, |s| s[1]), Some(None));
        assert_eq!(SLOT.with(&TLS, |s| s[2]), Some(Some(30)));
        assert_eq!(SLOT.take_lock_acquires(), 0);

        // Calling freeze again without changes preserves the OnceLock fast path.
        SLOT.freeze(|| panic!("should not rebuild unchanged store"));
        assert!(SLOT.is_once_active());
        assert_eq!(SLOT.take_lock_acquires(), 0);

        // Invalidate + re-record switches to TLS-cached fallback (1 acquire per thread).
        SLOT.invalidate();
        assert!(!SLOT.is_once_active());
        assert_eq!(SLOT.with(&TLS, |s| s[0]), None);
        SLOT.mark_dirty();
        SLOT.freeze(|| Some(vec![Some(99)].into_boxed_slice()));
        assert!(!SLOT.is_once_active());
        assert_eq!(SLOT.with(&TLS, |s| s[0]), Some(Some(99)));
        assert_eq!(SLOT.with(&TLS, |s| s[0]), Some(Some(99)));
        assert_eq!(SLOT.take_lock_acquires(), 1);
    }
}
