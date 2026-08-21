# FluxPay

> **High-performance payment orchestration engine built in Rust for reliable, fault-tolerant payment execution across multiple payment providers.**

**Rust • Actix Web • Tokio • PostgreSQL • Redis • SQLx • Docker • REST**

---

## Project Overview

FluxPay is a production-oriented backend that explores the core engineering challenges of modern payment infrastructure. It focuses on building a resilient payment orchestration layer capable of routing transactions across multiple providers while ensuring correctness, consistency, and reliability.

---

## Business Problem

Modern payment systems must handle failures without compromising transaction integrity.

FluxPay addresses challenges such as:

* Duplicate payment requests
* Provider outages
* Network timeouts
* Retry handling
* Transaction consistency
* Monetary precision

---

## Core Domain

### Payment Lifecycle

```
CREATED
    │
    ▼
PROCESSING
   ╱       ╲
  ▼         ▼
SUCCESS   FAILED
```

Every payment follows a deterministic lifecycle, ensuring only valid state transitions while preserving transactional consistency.

---

## System Capabilities

* RESTful payment API
* Multi-provider payment routing
* Provider abstraction
* Distributed idempotency with Redis
* Provider-aware retry strategy
* Exponential backoff with jitter
* Circuit breaker
* Automatic provider failover
* Persistent transaction storage
* Structured logging
* Explicit payment state machine

---

## Payment Processing Pipeline

```
Client Request
      │
      ▼
Validate Request
      │
      ▼
Idempotency Check (Redis)
      │
 ┌────┴─────┐
 │          │
Hit        Miss
 │          │
 ▼          ▼
Return    Persist Payment
Cached     (CREATED)
Response      │
              ▼
       Select Provider
              │
              ▼
      Circuit Breaker Check
              │
       ┌──────┴──────┐
       │             │
    Closed        Open
       │             │
       ▼             ▼
Provider Call    Failover /
       │          Next Provider
 ┌─────┴─────┐
 │           │
Success    Failure
 │           │
 ▼           ▼
Persist   Classify Error
SUCCESS       │
         ┌────┴─────┐
         │          │
      Retryable  Non-Retryable
         │          │
         ▼          ▼
  Exponential    Failover
  Backoff +       / Next
    Jitter       Provider
         │
         ▼
     Retry Call
         │
    ┌────┴─────┐
    │          │
 Success    Exhausted
    │          │
    ▼          ▼
Persist     Circuit
SUCCESS     Breaker
            Evaluation
                │
                ▼
             Failover
                │
                ▼
        Next Available Provider
                │
                ▼
          Final Transaction
              State
```

Every payment request is validated and protected by Redis-backed distributed idempotency before execution. The payment is persisted in the CREATED state and passed to the orchestration layer, which selects an appropriate provider and evaluates its circuit state before making the external call.

Transient provider failures are classified and retried according to the retry policy using exponential backoff and jitter, reducing the risk of synchronized retry storms. Repeated provider failures contribute to the circuit breaker, which can prevent further requests from being sent to an unhealthy provider.

When retries are exhausted, the failure is non-retryable, or the provider's circuit is open, the orchestrator can fail over to another available provider. The final transaction state is then persisted and returned to the client.
---


**Design Principles**

* Domain-Driven Design (DDD)
* Clean Architecture
* Repository Pattern
* Provider Abstraction
* Explicit Payment State Machine

---

## Technology Stack

| Layer            | Technology              |
| ---------------- | ----------------------- |
| Language         | Rust                    |
| Web Framework    | Actix Web               |
| Async Runtime    | Tokio                   |
| Database         | PostgreSQL              |
| Database Access  | SQLx                    |
| Cache            | Redis                   |
| API              | REST                    |
| Containerization | Docker & Docker Compose |
| Logging          | tracing                 |

---

## MVP Scope

### Included

* Payment creation
* Payment state machine
* Provider abstraction
* Retry & failover
* Distributed idempotency
* Payment persistence

### Out of Scope

* Refunds
* Authentication & authorization
* Webhooks
* Event streaming (Kafka/RabbitMQ)
* Ledger accounting

---

## Roadmap

* **Phase 2:** Refunds & transaction reversals
* **Phase 3:** Authentication & merchant isolation
* **Phase 4:** Transactional Outbox Pattern & Kafka integration
* **Phase 5:** OpenTelemetry, metrics, and distributed tracing

---

## Running the Project

### Prerequisites

* Rust (stable)
* Docker
* Docker Compose

# Clone repository
git@github.com:AraMinjibir/flux-pay.git
cd pay-flux

# Configure environment
cp .env.example .env
### Start Infrastructure

```bash
docker compose up -d
```

### Run Database Migrations

```bash
cargo sqlx database setup
```

### Start the Application

```bash
cargo run
```
