use std::time::Instant;

use actix_web::cookie::time::Duration;


#[derive(Debug)]
pub enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
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
    pub fn new(
        failure_threshold: u32,
        recovery_timeout: Duration,
    ) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            recovery_timeout,
            half_open_probe_in_progress: false,
        }
    }

    pub fn before_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,

            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.recovery_timeout {
                    self.state = CircuitState::HalfOpen;
                    self.half_open_probe_in_progress = true;

                    true
                } else {
                    false
                }
            }

            CircuitState::HalfOpen => {
                if self.half_open_probe_in_progress {
                    false
                } else {
                    self.half_open_probe_in_progress = true;

                    true
                }
            }
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.half_open_probe_in_progress = false;
    }

    pub fn record_failure(&mut self) {
        self.half_open_probe_in_progress = false;

        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;

                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                }
            }

            CircuitState::HalfOpen => {
                self.state = CircuitState::Open {
                    opened_at: Instant::now(),
                };
            }

            CircuitState::Open { .. } => {}
        }
    }
}