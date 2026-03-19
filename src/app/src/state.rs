#[cfg(feature = "ssr")]
pub mod ssr {
    use std::net::IpAddr;
    use std::sync::Arc;

    use dashmap::DashMap;
    use governor::{
        clock::DefaultClock,
        state::{InMemoryState, NotKeyed},
        RateLimiter,
    };
    use sqlx::SqlitePool;
    use tower_sessions_sqlx_store::SqliteStore;

    type IpLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

    #[derive(Clone)]
    pub struct WebAppState {
        pub pool: Option<SqlitePool>,
        pub session_store: SqliteStore,
        pub operation_locks: Arc<DashMap<i64, bool>>,
        pub rate_limiters: Arc<DashMap<IpAddr, IpLimiter>>,
        pub encryption_key: Option<Arc<net_ninja::crypto::EncryptionKey>>,
        pub settings: Arc<net_ninja::config::Settings>,
    }

    impl WebAppState {
        pub fn require_pool(&self) -> Result<&SqlitePool, String> {
            self.pool.as_ref().ok_or_else(|| "Database not available".to_string())
        }
    }
}

#[cfg(feature = "ssr")]
pub use ssr::WebAppState;
