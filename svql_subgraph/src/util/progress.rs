use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct Progress {
    total_candidates: AtomicU64,
    scanned_candidates: AtomicU64,
}

#[derive(Debug, Clone, Copy)]
pub struct ProgressSnapshot {
    pub total_candidates: u64,
    pub scanned_candidates: u64,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            total_candidates: AtomicU64::new(0),
            scanned_candidates: AtomicU64::new(0),
        }
    }
}

impl Progress {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set/overwrite the total number of candidates we expect to scan.
    /// Note: this is an estimate computed up‑front; it is fine if pruning
    /// reduces real scans below this number.
    pub fn set_total_candidates(&self, total: u64) {
        self.total_candidates.store(total, Ordering::Relaxed);
    }

    /// Add to the total (if you want to incrementally adjust).
    pub fn add_total_candidates(&self, delta: u64) {
        self.total_candidates.fetch_add(delta, Ordering::Relaxed);
    }

    /// Increment the number of candidates that have been scanned/considered.
    pub fn inc_scanned(&self, by: u64) {
        self.scanned_candidates.fetch_add(by, Ordering::Relaxed);
    }

    /// Read a consistent snapshot for display/reporting.
    pub fn snapshot(&self) -> ProgressSnapshot {
        ProgressSnapshot {
            total_candidates: self.total_candidates.load(Ordering::Relaxed),
            scanned_candidates: self.scanned_candidates.load(Ordering::Relaxed),
        }
    }
}
