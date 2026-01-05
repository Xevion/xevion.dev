use futures::future::{BoxFuture, FutureExt, Shared};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// The state of the health check system
enum HealthCheckState {
    /// No check has ever been performed
    Initial,

    /// A check is currently in progress, all requests await this future
    Checking {
        future: Shared<BoxFuture<'static, bool>>,
    },

    /// We have a cached result from a completed check
    Cached { healthy: bool, checked_at: Instant },
}

/// Inner state that can be shared across futures
struct HealthCheckerInner {
    state: Mutex<HealthCheckState>,
    had_success: AtomicBool,
}

/// Manages health check state with caching and singleflight behavior
pub struct HealthChecker {
    inner: Arc<HealthCheckerInner>,
    check_fn: Arc<dyn Fn() -> BoxFuture<'static, bool> + Send + Sync>,
}

impl HealthChecker {
    /// Create a new health checker with the given check function
    pub fn new<F, Fut>(check_fn: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = bool> + Send + 'static,
    {
        Self {
            inner: Arc::new(HealthCheckerInner {
                state: Mutex::new(HealthCheckState::Initial),
                had_success: AtomicBool::new(false),
            }),
            check_fn: Arc::new(move || check_fn().boxed()),
        }
    }

    /// Perform a health check with caching and singleflight behavior
    pub async fn check(&self) -> bool {
        let mut state = self.inner.state.lock().await;

        match &*state {
            HealthCheckState::Initial => {
                // Start first check, transition to Checking
                let future = self.create_check_future();
                *state = HealthCheckState::Checking {
                    future: future.clone(),
                };
                drop(state);
                future.await
            }
            HealthCheckState::Checking { future } => {
                // Join existing check (singleflight)
                let future = future.clone();
                drop(state);
                future.await
            }
            HealthCheckState::Cached {
                healthy,
                checked_at,
            } => {
                // Determine cache window based on startup status
                let window = if self.inner.had_success.load(Ordering::Relaxed) {
                    Duration::from_secs(15)
                } else {
                    Duration::from_secs(1)
                };

                if checked_at.elapsed() < window {
                    // Serve from cache
                    return *healthy;
                }

                // Cache stale, start new check
                let future = self.create_check_future();
                *state = HealthCheckState::Checking {
                    future: future.clone(),
                };
                drop(state);
                future.await
            }
        }
    }

    /// Create a shared future that performs the check and updates state
    fn create_check_future(&self) -> Shared<BoxFuture<'static, bool>> {
        let inner = Arc::clone(&self.inner);
        let check_fn = Arc::clone(&self.check_fn);

        async move {
            let result = (check_fn)().await;

            // Transition: Checking → Cached
            *inner.state.lock().await = HealthCheckState::Cached {
                healthy: result,
                checked_at: Instant::now(),
            };

            if result {
                inner.had_success.store(true, Ordering::Relaxed);
            }

            result
        }
        .boxed()
        .shared()
    }
}
