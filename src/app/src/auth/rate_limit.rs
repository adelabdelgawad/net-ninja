//! Per-IP login rate limiting using governor (GCRA, 5 failures / 60s).

use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};

use crate::state::WebAppState;

#[allow(dead_code)]
type IpLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Check whether the IP is within the rate limit.
/// Returns true if the request is allowed, false if rate-limited.
pub fn check_rate_limit(state: &WebAppState, ip: IpAddr) -> bool {
    let limiter = state
        .rate_limiters
        .entry(ip)
        .or_insert_with(|| {
            let quota = Quota::per_minute(NonZeroU32::new(5).unwrap());
            Arc::new(RateLimiter::direct(quota))
        })
        .clone();

    limiter.check().is_ok()
}
