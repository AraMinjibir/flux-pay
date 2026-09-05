use std::time::{Duration, Instant};
use tracing::info;

#[derive(Debug)]
pub enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitPermission {
    Allow,
    Probe,
    Reject,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    recovery_timeout: Duration,
    half_open_probe_in_progress: bool,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            recovery_timeout,
            half_open_probe_in_progress: false,
        }
    }

    pub fn before_request(&mut self) -> CircuitPermission {
        info!(
            "Circuit before_request: state={:?}, failure_count={}",
            self.state, self.failure_count
        );

        match self.state {
            CircuitState::Closed => CircuitPermission::Allow,

            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.recovery_timeout {
                    info!("Circuit OPEN → HALF_OPEN: recovery timeout elapsed");

                    self.state = CircuitState::HalfOpen;
                    self.half_open_probe_in_progress = true;

                    CircuitPermission::Probe
                } else {
                    info!("Circuit OPEN → rejecting request: recovery timeout not elapsed");

                    CircuitPermission::Reject
                }
            }

            CircuitState::HalfOpen => {
                if self.half_open_probe_in_progress {
                    info!("Circuit HALF_OPEN → probe already in progress, rejecting request");

                    CircuitPermission::Reject
                } else {
                    info!("Circuit HALF_OPEN → allowing probe");

                    self.half_open_probe_in_progress = true;

                    CircuitPermission::Probe
                }
            }
        }
    }

    pub fn record_success(&mut self) {
        info!("Circuit success → CLOSED");

        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.half_open_probe_in_progress = false;
    }

    pub fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;

                info!(
                    "Circuit failure recorded: count={}/{}",
                    self.failure_count, self.failure_threshold
                );

                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open {
                        opened_at: Instant::now(),
                    };

                    self.failure_count = 0;
                    self.half_open_probe_in_progress = false;

                    info!("Circuit CLOSED → OPEN");
                }
            }

            CircuitState::HalfOpen => {
                self.state = CircuitState::Open {
                    opened_at: Instant::now(),
                };

                self.half_open_probe_in_progress = false;

                info!("Circuit HALF_OPEN → OPEN");
            }

            CircuitState::Open { .. } => {
                info!("Circuit already OPEN; failure ignored");
            }
        }
    }
}
