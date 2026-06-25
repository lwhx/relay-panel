// v0.4.6: shared multi-target selection for all forwarders (TCP / WS / TLS / UDP).
//
// One `TargetSelector` is created per listener and shared (Arc) across all of
// that listener's connections / UDP sessions, so a round-robin cursor advances
// globally for the rule rather than per-connection.
//
// `order()` returns the list of target indices to TRY for a single new
// connection / session, in priority order. The caller connects to the first
// index that succeeds:
//   - First       → only index 0 (no fallback). A failed primary fails the
//                    connection; later targets are standby config only.
//   - Failover     → strict 0,1,2,…; always starts at the primary and falls
//                    through to the next on failure.
//   - RoundRobin   → starts at the next cursor position and wraps; a failed
//                    pick may try the remaining targets in ring order.
//
// v0.4.21: per-target circuit breaker. After 3 consecutive connect() failures
// a target is skipped for 30 seconds (TARGET_CIRCUIT_BREAK_SECS). A successful
// connect() resets the failure count and clears the breaker immediately.
// Circuit-broken targets are filtered from order() results, with fail-open:
// if ALL targets are in circuit break, the full order list is returned so
// the connection is not permanently blocked.

use relay_shared::protocol::LoadBalanceStrategy;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const TARGET_FAILURE_THRESHOLD: u32 = 3;
const TARGET_CIRCUIT_BREAK_SECS: u64 = 30;

/// Per-target health for circuit breaking. Uses atomics so concurrent
/// connections can report results without a Mutex.
#[derive(Debug)]
struct TargetHealth {
    failure_count: AtomicU32,
    circuit_until_ms: AtomicU64,
}

impl Default for TargetHealth {
    fn default() -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            circuit_until_ms: AtomicU64::new(0),
        }
    }
}

#[derive(Debug)]
pub struct TargetSelector {
    strategy: LoadBalanceStrategy,
    len: usize,
    cursor: AtomicUsize,
    health: Vec<TargetHealth>,
}

impl TargetSelector {
    pub fn new(strategy: LoadBalanceStrategy, len: usize) -> Self {
        let mut health = Vec::with_capacity(len);
        for _ in 0..len {
            health.push(TargetHealth::default());
        }
        Self {
            strategy,
            len,
            cursor: AtomicUsize::new(0),
            health,
        }
    }

    /// Report the result of a connect() attempt for target `idx`.
    ///
    /// - `success`: reset failure_count and circuit_until_ms.
    /// - `!success`: increment failure_count; if >= THRESHOLD, set
    ///   circuit_until_ms = now + CIRCUIT_BREAK_SECS.
    ///
    /// Out-of-bounds `idx` is silently ignored (no panic).
    pub fn report(&self, idx: usize, success: bool) {
        let Some(h) = self.health.get(idx) else {
            return;
        };
        if success {
            h.failure_count.store(0, Ordering::Relaxed);
            h.circuit_until_ms.store(0, Ordering::Relaxed);
        } else {
            let count = h.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
            if count >= TARGET_FAILURE_THRESHOLD {
                let now_ms = now_millis();
                let until = now_ms.saturating_add(TARGET_CIRCUIT_BREAK_SECS * 1000);
                h.circuit_until_ms.store(until, Ordering::Relaxed);
            }
        }
    }

    /// Whether target `idx` is currently in circuit break (should be skipped).
    fn is_circuit_open(&self, idx: usize) -> bool {
        let Some(h) = self.health.get(idx) else {
            return false;
        };
        let until = h.circuit_until_ms.load(Ordering::Relaxed);
        if until == 0 {
            return false;
        }
        now_millis() < until
    }

    /// The ordered target indices to attempt for ONE new connection / session.
    /// Empty when there are no targets.
    ///
    /// v0.4.21: targets currently in circuit break are filtered out. If ALL
    /// targets are in circuit break, fail-open returns the unfiltered order
    /// so the connection isn't permanently blocked.
    pub fn order(&self) -> Vec<usize> {
        if self.len == 0 {
            return Vec::new();
        }
        let candidates: Vec<usize> = match self.strategy {
            LoadBalanceStrategy::First => vec![0],
            LoadBalanceStrategy::Failover => (0..self.len).collect(),
            LoadBalanceStrategy::RoundRobin => {
                let start = self.cursor.fetch_add(1, Ordering::Relaxed) % self.len;
                (0..self.len).map(|i| (start + i) % self.len).collect()
            }
        };
        let alive: Vec<usize> = candidates
            .iter()
            .copied()
            .filter(|&i| !self.is_circuit_open(i))
            .collect();
        if alive.is_empty() {
            candidates
        } else {
            alive
        }
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_targets_yield_no_order() {
        let s = TargetSelector::new(LoadBalanceStrategy::RoundRobin, 0);
        assert!(s.order().is_empty());
    }

    #[test]
    fn first_only_tries_primary() {
        let s = TargetSelector::new(LoadBalanceStrategy::First, 3);
        assert_eq!(s.order(), vec![0]);
        assert_eq!(s.order(), vec![0], "First never advances");
    }

    #[test]
    fn failover_is_strict_priority_from_primary() {
        let s = TargetSelector::new(LoadBalanceStrategy::Failover, 3);
        assert_eq!(s.order(), vec![0, 1, 2]);
        assert_eq!(
            s.order(),
            vec![0, 1, 2],
            "Failover always starts at primary"
        );
    }

    #[test]
    fn round_robin_advances_and_wraps() {
        let s = TargetSelector::new(LoadBalanceStrategy::RoundRobin, 3);
        assert_eq!(s.order(), vec![0, 1, 2]);
        assert_eq!(s.order(), vec![1, 2, 0]);
        assert_eq!(s.order(), vec![2, 0, 1]);
        assert_eq!(s.order(), vec![0, 1, 2], "wraps back to primary");
    }

    #[test]
    fn round_robin_single_target_is_stable() {
        let s = TargetSelector::new(LoadBalanceStrategy::RoundRobin, 1);
        assert_eq!(s.order(), vec![0]);
        assert_eq!(s.order(), vec![0]);
    }

    // --- v0.4.21: circuit-breaker tests ---

    #[test]
    fn report_success_resets_failure_count() {
        let s = TargetSelector::new(LoadBalanceStrategy::Failover, 3);
        s.report(0, false);
        s.report(0, false);
        s.report(0, true);
        assert_eq!(s.order(), vec![0, 1, 2]);
    }

    #[test]
    fn three_failures_triggers_circuit() {
        let s = TargetSelector::new(LoadBalanceStrategy::Failover, 3);
        s.report(0, false);
        s.report(0, false);
        s.report(0, false);
        let order = s.order();
        assert!(
            !order.contains(&0),
            "target 0 should be circuit-broken, got {:?}",
            order
        );
        assert_eq!(order, vec![1, 2]);
    }

    #[test]
    fn circuit_expires_target_rejoins_after_30s() {
        let s = TargetSelector::new(LoadBalanceStrategy::Failover, 3);
        s.health[0]
            .circuit_until_ms
            .store(now_millis().saturating_sub(1000), Ordering::Relaxed);
        assert_eq!(s.order(), vec![0, 1, 2]);
    }

    #[test]
    fn all_targets_circuit_open_fail_open() {
        let s = TargetSelector::new(LoadBalanceStrategy::Failover, 3);
        for i in 0..3 {
            s.report(i, false);
            s.report(i, false);
            s.report(i, false);
        }
        let order = s.order();
        assert!(!order.is_empty(), "must fail-open, not return empty list");
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn report_out_of_bounds_ignored() {
        let s = TargetSelector::new(LoadBalanceStrategy::First, 1);
        s.report(1, false);
        s.report(5, true);
        s.report(usize::MAX, false);
        assert_eq!(s.order(), vec![0]);
    }

    #[test]
    fn first_always_returns_index_zero_even_when_circuit() {
        let s = TargetSelector::new(LoadBalanceStrategy::First, 3);
        s.report(0, false);
        s.report(0, false);
        s.report(0, false);
        assert_eq!(s.order(), vec![0]);
    }

    #[test]
    fn round_robin_skips_circuit_target() {
        let s = TargetSelector::new(LoadBalanceStrategy::RoundRobin, 3);
        assert_eq!(s.order(), vec![0, 1, 2]);
        s.report(0, false);
        s.report(0, false);
        s.report(0, false);
        let order = s.order();
        assert!(!order.contains(&0));
        assert_eq!(order, vec![1, 2]);
    }

    #[test]
    fn failover_skips_circuit_primary() {
        let s = TargetSelector::new(LoadBalanceStrategy::Failover, 3);
        s.report(0, false);
        s.report(0, false);
        s.report(0, false);
        assert_eq!(s.order(), vec![1, 2]);
    }

    #[test]
    fn concurrent_reports_no_panic() {
        use std::sync::Arc;
        use std::thread;
        let s = Arc::new(TargetSelector::new(LoadBalanceStrategy::RoundRobin, 5));
        let mut handles = vec![];
        for t in 0..10 {
            let s = s.clone();
            handles.push(thread::spawn(move || {
                for i in 0..5 {
                    s.report(i, false);
                    let _ = s.order();
                }
                s.report(t % 5, true);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let order = s.order();
        assert!(!order.is_empty());
    }
}
