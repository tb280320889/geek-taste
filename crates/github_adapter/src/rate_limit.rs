//! 速率预算管理器 — core(5000/h) + search(30/min) 端点隔离

use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use thiserror::Error;

/// 速率池类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatePool {
    /// 核心 REST API — 5000 req/h
    Core,
    /// Search API — 30 req/min
    Search,
}

/// 单池状态
struct PoolState {
    remaining: u32,
    resets_at: DateTime<Utc>,
}

/// 速率预算错误
#[derive(Debug, Error)]
pub enum RateError {
    #[error("Core API 速率限制，重置于 {0}")]
    CoreExceeded(DateTime<Utc>),
    #[error("Search API 速率限制，重置于 {0}")]
    SearchExceeded(DateTime<Utc>),
}

/// 速率预算管理器 — core(5000/h) + search(30/min) 端点隔离
pub struct RateBudget {
    core: Mutex<PoolState>,
    search: Mutex<PoolState>,
}

impl RateBudget {
    pub fn new() -> Self {
        Self {
            core: Mutex::new(PoolState {
                remaining: 5000,
                resets_at: Utc::now() + Duration::hours(1),
            }),
            search: Mutex::new(PoolState {
                remaining: 30,
                resets_at: Utc::now() + Duration::minutes(1),
            }),
        }
    }

    /// 检查是否可发起请求，返回错误信息若超限
    pub fn check(&self, pool: RatePool) -> Result<(), RateError> {
        let state = match pool {
            RatePool::Core => self.core.lock().expect("core mutex poisoned"),
            RatePool::Search => self.search.lock().expect("search mutex poisoned"),
        };
        let now = Utc::now();

        // 如果已过重置时间，自动恢复
        if now >= state.resets_at {
            drop(state);
            self.reset_pool(pool);
            return Ok(());
        }

        if state.remaining == 0 {
            return Err(match pool {
                RatePool::Core => RateError::CoreExceeded(state.resets_at),
                RatePool::Search => RateError::SearchExceeded(state.resets_at),
            });
        }

        Ok(())
    }

    /// 记录一次请求消耗
    pub fn record(&self, pool: RatePool) {
        let mut state = match pool {
            RatePool::Core => self.core.lock().expect("core mutex poisoned"),
            RatePool::Search => self.search.lock().expect("search mutex poisoned"),
        };

        let now = Utc::now();
        // 如果已过重置时间，先重置
        if now >= state.resets_at {
            match pool {
                RatePool::Core => {
                    state.remaining = 5000;
                    state.resets_at = now + Duration::hours(1);
                }
                RatePool::Search => {
                    state.remaining = 30;
                    state.resets_at = now + Duration::minutes(1);
                }
            }
        }

        if state.remaining > 0 {
            state.remaining -= 1;
        }
    }

    /// 从 GitHub 响应头更新剩余配额
    pub fn update_from_headers(&self, pool: RatePool, remaining: u32, resets_at: DateTime<Utc>) {
        let mut state = match pool {
            RatePool::Core => self.core.lock().expect("core mutex poisoned"),
            RatePool::Search => self.search.lock().expect("search mutex poisoned"),
        };
        state.remaining = remaining;
        state.resets_at = resets_at;
    }

    /// 内部辅助：重置池状态
    fn reset_pool(&self, pool: RatePool) {
        let mut state = match pool {
            RatePool::Core => self.core.lock().expect("core mutex poisoned"),
            RatePool::Search => self.search.lock().expect("search mutex poisoned"),
        };
        let now = Utc::now();
        match pool {
            RatePool::Core => {
                state.remaining = 5000;
                state.resets_at = now + Duration::hours(1);
            }
            RatePool::Search => {
                state.remaining = 30;
                state.resets_at = now + Duration::minutes(1);
            }
        }
    }

    /// 获取指定池剩余配额（用于 UI 展示）
    pub fn remaining(&self, pool: RatePool) -> u32 {
        let state = match pool {
            RatePool::Core => self.core.lock().expect("core mutex poisoned"),
            RatePool::Search => self.search.lock().expect("search mutex poisoned"),
        };
        if Utc::now() >= state.resets_at {
            match pool {
                RatePool::Core => 5000,
                RatePool::Search => 30,
            }
        } else {
            state.remaining
        }
    }
}

impl Default for RateBudget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_budget_has_full_quota() {
        let budget = RateBudget::new();
        assert!(budget.check(RatePool::Core).is_ok());
        assert!(budget.check(RatePool::Search).is_ok());
    }

    #[test]
    fn record_decrements_remaining() {
        let budget = RateBudget::new();
        budget.record(RatePool::Search);
        assert_eq!(budget.remaining(RatePool::Search), 29);
    }

    #[test]
    fn check_returns_error_when_exhausted() {
        let budget = RateBudget::new();
        // Manually set remaining to 0 via update_from_headers
        let past = Utc::now() + Duration::hours(1);
        budget.update_from_headers(RatePool::Search, 0, past);

        let err = budget.check(RatePool::Search);
        assert!(err.is_err());
        match err {
            Err(RateError::SearchExceeded(_)) => {}
            _ => panic!("expected SearchExceeded"),
        }
    }

    #[test]
    fn core_exhausted_returns_core_error() {
        let budget = RateBudget::new();
        let past = Utc::now() + Duration::hours(1);
        budget.update_from_headers(RatePool::Core, 0, past);

        match budget.check(RatePool::Core) {
            Err(RateError::CoreExceeded(_)) => {}
            _ => panic!("expected CoreExceeded"),
        }
    }

    #[test]
    fn pools_are_independent() {
        let budget = RateBudget::new();
        // Exhaust search pool
        let future = Utc::now() + Duration::minutes(5);
        budget.update_from_headers(RatePool::Search, 0, future);

        // Core pool should still work
        assert!(budget.check(RatePool::Core).is_ok());
        // Search pool should be exhausted
        assert!(budget.check(RatePool::Search).is_err());
    }

    #[test]
    fn update_from_headers_syncs_state() {
        let budget = RateBudget::new();
        let resets = Utc::now() + Duration::minutes(30);
        budget.update_from_headers(RatePool::Core, 2500, resets);
        assert_eq!(budget.remaining(RatePool::Core), 2500);
    }

    #[test]
    fn record_respects_zero_remaining() {
        let budget = RateBudget::new();
        let future = Utc::now() + Duration::hours(1);
        budget.update_from_headers(RatePool::Search, 0, future);
        // record on exhausted pool should not panic
        budget.record(RatePool::Search);
        assert_eq!(budget.remaining(RatePool::Search), 0);
    }
}
