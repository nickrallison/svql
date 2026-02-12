use crate::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

/// A OnceLock with wait capability for blocking on unfilled slots.
///
/// Properties:
/// - Lock-free reads when filled (via OnceLock)
/// - Blocking wait when being filled by another thread (via Condvar)
/// - Single-writer guarantee (via CAS on `claimed`)
///
/// The execution claim lives here (not on `ExecutionNode`) so that
/// it is keyed by `TypeId` in the slots map. Multiple `ExecutionNode`
/// copies for the same pattern type all share this single slot,
/// giving a correct single-execution guarantee.
pub struct TableSlot {
    /// The actual data - written once by the winning thread
    data: OnceLock<Arc<dyn AnyTable + Send + Sync>>,
    /// CAS flag: whoever flips false→true owns execution
    claimed: AtomicBool,
    /// Condition variable for waiting threads
    cvar: Condvar,
    /// Minimal lock - ONLY used for condvar wait predicate, not data access
    lock: Mutex<()>,
}

/// Result of attempting to claim a slot for execution.
pub enum ClaimResult<'a> {
    /// Already filled — here is the data (lock-free fast path).
    Ready(&'a (dyn AnyTable + Send + Sync)),
    /// You won the CAS — execute the search and call `set()`.
    Claimed,
    /// Another thread is executing — block until `set()` is called.
    Wait,
}

impl TableSlot {
    /// Creates a new empty slot for a result table.
    pub fn new() -> Self {
        Self {
            data: OnceLock::new(),
            claimed: AtomicBool::new(false),
            cvar: Condvar::new(),
            lock: Mutex::new(()),
        }
    }

    /// Attempt to claim this slot for execution.
    ///
    /// Three possible outcomes:
    /// - `Ready`: slot already filled (lock-free read)
    /// - `Claimed`: caller won the race — must execute and call `set()`
    /// - `Wait`: another thread is executing — caller should call `wait()`
    pub fn try_claim(&self) -> ClaimResult<'_> {
        // Fast path: already filled (lock-free)
        if let Some(arc) = self.data.get() {
            return ClaimResult::Ready(arc.as_ref());
        }

        // Try to claim execution rights
        let was_unclaimed = self
            .claimed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();

        if was_unclaimed {
            ClaimResult::Claimed
        } else {
            ClaimResult::Wait
        }
    }

    /// Try to get the table (non-blocking).
    ///
    /// Returns `Some(Arc<...>)` if already filled, `None` if empty or being filled.
    /// This is a lock-free fast path when the table is ready.
    pub fn get(&self) -> Option<Arc<dyn AnyTable + Send + Sync>> {
        self.data.get().map(Arc::clone)
    }

    /// Wait for the table to be filled and return it (blocking).
    ///
    /// Used by threads that lost the CAS race.
    /// Blocks until another thread calls `set()`.
    pub fn wait(&self) -> Arc<dyn AnyTable + Send + Sync> {
        // Fast path: already filled (common case after first access)
        if let Some(arc) = self.data.get() {
            return Arc::clone(arc);
        }

        // Slow path: need to wait for another thread to fill it
        let mut guard = self.lock.lock().unwrap();
        loop {
            // Re-check inside the lock (standard condvar pattern)
            if let Some(arc) = self.data.get() {
                return Arc::clone(arc);
            }
            // Wait for notification from set()
            guard = self.cvar.wait(guard).unwrap();
        }
    }

    /// Set the table value (should only be called once).
    ///
    /// Called by the thread that won the CAS race.
    /// Wakes up all threads waiting in `wait()`.
    pub fn set(&self, value: Arc<dyn AnyTable + Send + Sync>) {
        // Store the data (OnceLock ensures single write)
        let _ = self.data.set(value);

        // Wake up all waiting threads
        // We must lock before notify to prevent lost wakeups
        let _guard = self.lock.lock().unwrap();
        self.cvar.notify_all();
    }

    /// Get a reference to the table without cloning the Arc.
    ///
    /// This is used when we need a `&(dyn AnyTable)` reference with a lifetime
    /// tied to the slot itself rather than a temporary Arc.
    pub fn get_ref(&self) -> Option<&(dyn AnyTable + Send + Sync)> {
        self.data.get().map(|arc| arc.as_ref())
    }
}

impl Default for TableSlot {
    fn default() -> Self {
        Self::new()
    }
}
