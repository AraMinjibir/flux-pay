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
* Distributed idempotency with Redis
* Automatic retry strategy
* Provider failover
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
  ┌─────┴─────┐
  │           │
Hit          Miss
  │           │
  ▼           ▼
Return     Persist Payment
Cached      (CREATED)
Response         │
                 ▼
        Primary Provider
                 │
        ┌────────┴────────┐
        │                 │
    Success           Failure
        │                 │
        ▼                 ▼
 Persist SUCCESS      Retry Once
                          │
                 ┌────────┴────────┐
                 │                 │
             Success           Failure
                 │                 │
                 ▼                 ▼
        Persist SUCCESS   Secondary Provider
                                   │
                          ┌────────┴────────┐
                          │                 │
                      Success           Failure
                          │                 │
                          ▼                 ▼
                 Persist SUCCESS   Persist FAILED
                          │
                          ▼
                   HTTP Response
```

Every request is validated, protected by Redis-backed idempotency, persisted before execution, and processed through a resilient orchestration pipeline. Transient failures trigger retries, while unrecoverable failures automatically fail over to a secondary payment provider before the final transaction state is recorded.

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
