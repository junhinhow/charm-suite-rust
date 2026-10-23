//! Virtual clock.

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::lock;

/// Minimum clock interface used by models: the current instant and the duration
/// elapsed since another instant.
pub trait Clock {
    /// Returns the current instant.
    fn now(&self) -> SystemTime;
    /// Returns how much time has elapsed since `t`.
    fn since(&self, t: SystemTime) -> Duration;
}

const MAX_TICKS: usize = 1 << 20;

struct FakeClockInner {
    start: SystemTime,
    now: SystemTime,
}

/// A virtual clock: time only changes through [`FakeClock::advance`] or
/// [`FakeClock::set`], never through the machine clock.
#[derive(Clone)]
pub struct FakeClock {
    inner: Arc<Mutex<FakeClockInner>>,
}

impl std::fmt::Debug for FakeClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeClock").finish_non_exhaustive()
    }
}

impl FakeClock {
    /// Creates a clock stopped at `start`.
    pub fn new(start: SystemTime) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FakeClockInner { start, now: start })),
        }
    }

    /// Returns the current virtual instant.
    pub fn now(&self) -> SystemTime {
        lock(&self.inner).now
    }

    /// Returns the duration between `t` and the current virtual instant.
    pub fn since(&self, t: SystemTime) -> Duration {
        self.now().duration_since(t).unwrap_or(Duration::ZERO)
    }

    /// Adds `d` to the current instant.
    ///
    /// Zero durations are ignored: the clock never moves backwards through this
    /// method. Use [`FakeClock::set`] to jump backwards.
    pub fn advance(&self, d: Duration) {
        if d.is_zero() {
            return;
        }
        let mut inner = lock(&self.inner);
        inner.now += d;
    }

    /// Moves the clock to `t`, including backwards.
    ///
    /// The creation instant used by [`FakeClock::ticks`] does not change.
    pub fn set(&self, t: SystemTime) {
        lock(&self.inner).now = t;
    }

    /// Returns the complete `d`-sized instants elapsed since creation, excluding
    /// the creation instant itself.
    ///
    /// Returns `None` when `d` is zero or negative, when the clock has not
    /// advanced, or when it moved backwards.
    pub fn ticks(&self, d: Duration) -> Option<Vec<SystemTime>> {
        if d.is_zero() {
            return None;
        }
        let (start, now) = {
            let inner = lock(&self.inner);
            (inner.start, inner.now)
        };
        let elapsed = now.duration_since(start).ok()?;
        if elapsed.is_zero() {
            return None;
        }
        let n_nanos = elapsed.as_nanos() / d.as_nanos();
        if n_nanos == 0 {
            return None;
        }
        let mut n = n_nanos as usize;
        if n > MAX_TICKS {
            n = MAX_TICKS;
        }
        let mut ticks = Vec::with_capacity(n);
        let mut t = start;
        for _ in 0..n {
            t = t.checked_add(d)?;
            ticks.push(t);
        }
        Some(ticks)
    }
}

impl Clock for FakeClock {
    fn now(&self) -> SystemTime {
        self.now()
    }

    fn since(&self, t: SystemTime) -> Duration {
        self.since(t)
    }
}
