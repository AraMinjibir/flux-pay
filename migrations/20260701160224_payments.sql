CREATE TABLE payments (
    id UUID PRIMARY KEY,

    merchant_id UUID NOT NULL,

    amount BIGINT NOT NULL,
    currency CHAR(3) NOT NULL,

    description TEXT,

    reference VARCHAR(255) NOT NULL UNIQUE,

    status VARCHAR(20) NOT NULL,
    payment_method VARCHAR(50) NOT NULL,
    payment_provider VARCHAR(50) NOT NULL,

    provider_reference VARCHAR(255),

    failure_reason TEXT,
    retry_count SMALLINT NOT NULL DEFAULT 0,

    idempotency_key UUID  UNIQUE,

    paid_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_payments_merchant_id
ON payments (merchant_id);

CREATE INDEX idx_payments_status
ON payments (status);

CREATE INDEX idx_payments_paid_at
ON payments (paid_at);

CREATE INDEX idx_payments_provider
ON payments (payment_provider);

CREATE INDEX idx_payments_merchant_paid_at
ON payments (merchant_id, paid_at DESC);