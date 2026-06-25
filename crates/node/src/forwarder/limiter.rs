// v0.4.6: per-rule, bidirectional, connection-shared rate limiting.
//
// One `RuleLimiter` covers BOTH the upload and download directions of a single
// rule, and is shared (Arc) across ALL of that rule's connections / UDP
// sessions / TCP+UDP listeners. This is deliberate: a `tcp_udp` rule must NOT
// get double the budget, and the cap is on the RULE's aggregate traffic, not
// per-connection.
//
// Implementation: a classic continuously-refilled token bucket per direction,
// refilled lazily on each `acquire()` (elapsed time × rate). When the requested
// amount exceeds the current balance, the caller sleeps for the deficit time
// and then consumes. This keeps steady-state throughput at exactly the cap and
// allows short bursts up to the bucket capacity (= 1 second of traffic).
//
// A rate of 0 / None means unlimited: `acquire()` is a no-op, so unlimited
// rules pay only a single branch per chunk.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

/// One second of burst capacity — lets a connection send a full-rate chunk
/// immediately, then throttle back to the refill rate.
const BURST_SECS: u64 = 1;

struct Bucket {
    /// Refill rate in bytes/sec. 0 = this direction is unlimited.
    rate_bps: u64,
    /// Current token balance (capped at rate_bps * BURST_SECS).
    tokens: u64,
    last_refill: Instant,
}

impl Bucket {
    fn new(rate_bps: u64) -> Self {
        Self {
            rate_bps,
            // Start full so the first chunk isn't artificially delayed.
            tokens: rate_bps.saturating_mul(BURST_SECS),
            last_refill: Instant::now(),
        }
    }

    /// Refill based on elapsed wall time, then consume `want` bytes (sleeping
    /// if the balance is too low). Returns the bytes actually reserved (always
    /// == want once the await completes).
    async fn acquire(&mut self, want: u64) {
        if self.rate_bps == 0 {
            return;
        }
        let cap = self.rate_bps.saturating_mul(BURST_SECS);
        loop {
            let now = Instant::now();
            let elapsed = now.duration_since(self.last_refill).as_secs_f64();
            // Lazily refill, capped at the burst ceiling.
            self.tokens = (self.tokens as f64 + elapsed * self.rate_bps as f64) as u64;
            if self.tokens > cap {
                self.tokens = cap;
            }
            self.last_refill = now;

            if self.tokens >= want {
                self.tokens -= want;
                return;
            }
            // Not enough: sleep for the time needed to earn the remaining bytes,
            // then loop to recompute (avoids underflow on u64).
            let deficit = want - self.tokens;
            let wait = Duration::from_secs_f64(deficit as f64 / self.rate_bps as f64);
            tokio::time::sleep(wait).await;
        }
    }
}

/// Per-rule rate limiter. Shared across all connections of the rule.
pub struct RuleLimiter {
    upload: Mutex<Bucket>,
    download: Mutex<Bucket>,
    /// Cached so callers can short-circuit without locking when both are zero.
    unlimited: bool,
}

impl RuleLimiter {
    /// `upload_bps` / `download_bps` are bytes/sec; None / 0 means unlimited.
    pub fn new(upload_bps: Option<u64>, download_bps: Option<u64>) -> Self {
        let up = upload_bps.unwrap_or(0);
        let down = download_bps.unwrap_or(0);
        Self {
            upload: Mutex::new(Bucket::new(up)),
            download: Mutex::new(Bucket::new(down)),
            unlimited: up == 0 && down == 0,
        }
    }

    #[allow(dead_code)]
    pub fn unlimited() -> Self {
        Self::new(None, None)
    }

    #[allow(dead_code)]
    pub fn is_unlimited(&self) -> bool {
        self.unlimited
    }

    /// Reserve `n` upload (client→target) bytes. No-op for unlimited limiters.
    pub async fn acquire_upload(&self, n: u64) {
        if self.unlimited {
            return;
        }
        self.upload.lock().await.acquire(n).await;
    }

    /// Reserve `n` download (target→client) bytes. No-op for unlimited limiters.
    pub async fn acquire_download(&self, n: u64) {
        if self.unlimited {
            return;
        }
        self.download.lock().await.acquire(n).await;
    }
}

/// A handle that is either a real shared limiter or "no limit" (the common
/// case for v0.4.5 rules and any rule with 0 caps). Cloned cheaply into every
/// connection / UDP session task of a rule.
#[derive(Clone)]
pub enum RateLimit {
    Unlimited,
    Limited(Arc<RuleLimiter>),
}

impl RateLimit {
    /// Build the shared limiter for a listener's (rule_id, direction) caps.
    /// `None`/0 on both sides → Unlimited (zero per-connection overhead).
    pub fn new(upload_bps: Option<u64>, download_bps: Option<u64>) -> Self {
        if upload_bps.unwrap_or(0) == 0 && download_bps.unwrap_or(0) == 0 {
            RateLimit::Unlimited
        } else {
            RateLimit::Limited(Arc::new(RuleLimiter::new(upload_bps, download_bps)))
        }
    }

    pub async fn acquire_upload(&self, n: u64) {
        if let RateLimit::Limited(l) = self {
            l.acquire_upload(n).await;
        }
    }
    pub async fn acquire_download(&self, n: u64) {
        if let RateLimit::Limited(l) = self {
            l.acquire_download(n).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unlimited_is_instant_noop() {
        let r = RateLimit::new(None, None);
        assert!(matches!(r, RateLimit::Unlimited));
        // Should return immediately even for a huge amount.
        let start = Instant::now();
        r.acquire_upload(u64::MAX / 2).await;
        r.acquire_download(u64::MAX / 2).await;
        assert!(start.elapsed() < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn acquire_blocks_then_refills() {
        // 1000 byte/s. Requesting 1000 bytes right away consumes the initial
        // full bucket; requesting another 1000 immediately must wait ~1s.
        let r = RateLimit::new(Some(1000), Some(1000));
        r.acquire_upload(1000).await; // drains burst
        let start = Instant::now();
        r.acquire_upload(1000).await; // must refill ~1s
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(900),
            "expected ~1s wait, got {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn shared_bucket_aggregates_across_clone() {
        // Two tasks sharing one limiter must draw from the SAME bucket, so the
        // combined rate is capped, not doubled.
        let r = RateLimit::new(Some(1000), Some(0));
        // Drain the initial burst once.
        r.acquire_upload(1000).await;
        let r2 = r.clone();
        let h1 = tokio::spawn(async move { r.acquire_upload(500).await });
        let h2 = tokio::spawn(async move { r2.acquire_upload(500).await });
        let start = Instant::now();
        h1.await.unwrap();
        h2.await.unwrap();
        // Together they requested 1000 bytes at 1000 byte/s → ~1s total (the
        // two halves may serialize on the bucket lock, but the refill time is
        // bounded by the aggregate amount).
        assert!(start.elapsed() >= Duration::from_millis(900));
    }
}
