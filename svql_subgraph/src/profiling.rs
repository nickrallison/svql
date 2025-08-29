// use std::time::{Duration, Instant};

#[cfg(feature = "profiling")]
mod enabled {
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};

    use dashmap::DashMap;

    type Count = u64;
    type TotalNanos = u128;

    static STATS: OnceLock<DashMap<&'static str, (Count, TotalNanos)>> = OnceLock::new();

    #[inline]
    fn stats() -> &'static DashMap<&'static str, (Count, TotalNanos)> {
        STATS.get_or_init(DashMap::new)
    }

    pub struct Timer {
        label: &'static str,
        start: Instant,
    }

    impl Timer {
        #[inline]
        pub fn new(label: &'static str) -> Self {
            Timer {
                label,
                start: Instant::now(),
            }
        }
    }

    impl Drop for Timer {
        fn drop(&mut self) {
            let dur = self.start.elapsed();
            let nanos = dur.as_nanos();

            let mut entry = stats().entry(self.label).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += nanos;
        }
    }

    pub fn record(label: &'static str, dur: Duration) {
        let nanos = dur.as_nanos();
        let mut entry = stats().entry(label).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += nanos;
    }

    pub fn report() {
        let mut rows: Vec<(&'static str, Count, TotalNanos, f64)> = stats()
            .iter()
            .map(|kv| {
                let (label, (count, total)) = (kv.key(), *kv.value());
                let avg_ns = if count == 0 {
                    0.0
                } else {
                    (total as f64) / (count as f64)
                };
                (*label, count, total, avg_ns)
            })
            .collect();

        rows.sort_by(|a, b| b.2.cmp(&a.2)); // sort by total nanos desc

        println!("==== svql_subgraph index lookup profile ====");
        for (label, count, total_ns, avg_ns) in rows {
            let total_ms = (total_ns as f64) / 1_000_000.0;
            println!(
                "{:<60} calls={:<10} total={:>9.3} ms avg= {:>12.1} ns",
                label, count, total_ms, avg_ns
            );
        }
        println!("============================================");
    }
}

#[cfg(not(feature = "profiling"))]
mod disabled {
    use std::time::Duration;

    #[derive(Clone, Copy)]
    pub struct Timer;

    impl Timer {
        #[inline]
        pub fn new(_: &'static str) -> Self {
            Timer
        }
    }

    pub fn record(_: &'static str, _: Duration) {}
    pub fn report() {}
}

#[cfg(feature = "profiling")]
pub use enabled::*;

#[cfg(not(feature = "profiling"))]
pub use disabled::*;
