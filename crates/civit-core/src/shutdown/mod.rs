#![forbid(unsafe_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub struct ShutdownManager {
    shutdown_requested: Arc<AtomicBool>,
    timeout: Duration,
}

impl ShutdownManager {
    pub fn new(timeout: Duration) -> Self {
        Self {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            timeout,
        }
    }

    pub fn request_shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
    }

    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn subscribe(&self) -> ShutdownSignal {
        ShutdownSignal {
            flag: Arc::clone(&self.shutdown_requested),
        }
    }
}

pub struct ShutdownSignal {
    flag: Arc<AtomicBool>,
}

impl ShutdownSignal {
    pub fn is_triggered(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    pub fn wait(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.is_triggered() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }
}

impl Clone for ShutdownSignal {
    fn clone(&self) -> Self {
        Self {
            flag: Arc::clone(&self.flag),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_manager_initial_state() {
        let mgr = ShutdownManager::new(Duration::from_secs(30));
        assert!(!mgr.is_shutdown_requested());
        assert_eq!(mgr.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_shutdown_manager_request() {
        let mgr = ShutdownManager::new(Duration::from_secs(10));
        mgr.request_shutdown();
        assert!(mgr.is_shutdown_requested());
    }

    #[test]
    fn test_shutdown_signal_not_triggered_initially() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        let signal = mgr.subscribe();
        assert!(!signal.is_triggered());
    }

    #[test]
    fn test_shutdown_signal_triggered_after_request() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        let signal = mgr.subscribe();
        mgr.request_shutdown();
        assert!(signal.is_triggered());
    }

    #[test]
    fn test_shutdown_signal_wait_triggered() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        let signal = mgr.subscribe();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            mgr.request_shutdown();
        });
        let result = signal.wait(Duration::from_secs(1));
        assert!(result);
    }

    #[test]
    fn test_shutdown_signal_wait_timeout() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        let signal = mgr.subscribe();
        let result = signal.wait(Duration::from_millis(50));
        assert!(!result);
    }

    #[test]
    fn test_multiple_subscribers() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        let sig1 = mgr.subscribe();
        let sig2 = mgr.subscribe();
        assert!(!sig1.is_triggered());
        assert!(!sig2.is_triggered());
        mgr.request_shutdown();
        assert!(sig1.is_triggered());
        assert!(sig2.is_triggered());
    }

    #[test]
    fn test_signal_clone() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        let sig = mgr.subscribe();
        let sig_clone = sig.clone();
        mgr.request_shutdown();
        assert!(sig_clone.is_triggered());
    }

    #[test]
    fn test_shutdown_manager_timeout_values() {
        let mgr = ShutdownManager::new(Duration::from_millis(500));
        assert_eq!(mgr.timeout(), Duration::from_millis(500));

        let mgr = ShutdownManager::new(Duration::from_secs(0));
        assert_eq!(mgr.timeout(), Duration::from_secs(0));

        let mgr = ShutdownManager::new(Duration::from_secs(3600));
        assert_eq!(mgr.timeout(), Duration::from_secs(3600));
    }

    #[test]
    fn test_signal_wait_with_zero_timeout() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        let signal = mgr.subscribe();
        let result = signal.wait(Duration::from_secs(0));
        assert!(!result);
    }

    #[test]
    fn test_request_shutdown_idempotent() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        mgr.request_shutdown();
        mgr.request_shutdown();
        assert!(mgr.is_shutdown_requested());
    }

    #[test]
    fn test_multiple_managers_independent() {
        let mgr1 = ShutdownManager::new(Duration::from_secs(1));
        let mgr2 = ShutdownManager::new(Duration::from_secs(2));
        let sig1 = mgr1.subscribe();
        let sig2 = mgr2.subscribe();
        mgr1.request_shutdown();
        assert!(sig1.is_triggered());
        assert!(!sig2.is_triggered());
    }

    #[test]
    fn test_signal_wait_immediate_trigger() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        mgr.request_shutdown();
        let signal = mgr.subscribe();
        assert!(signal.wait(Duration::from_millis(100)));
    }

    #[test]
    fn test_signal_wait_returns_early_on_trigger() {
        let mgr = ShutdownManager::new(Duration::from_secs(5));
        let signal = mgr.subscribe();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            mgr.request_shutdown();
        });
        let start = std::time::Instant::now();
        let result = signal.wait(Duration::from_secs(10));
        let elapsed = start.elapsed();
        assert!(result);
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn test_shutdown_manager_new_with_various_durations() {
        for ms in [0u64, 1, 100, 1000, 30_000, 3_600_000] {
            let mgr = ShutdownManager::new(Duration::from_millis(ms));
            assert_eq!(mgr.timeout(), Duration::from_millis(ms));
            assert!(!mgr.is_shutdown_requested());
        }
    }
}
