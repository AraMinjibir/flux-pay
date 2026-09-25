CREATE TABLE refunds (
    id UUID PRIMARY KEY,
    payment_id UUID NOT NULL,
    amount BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL,
    status VARCHAR(20) NOT NULL,
    reason TEXT,
    provider_refund_id VARCHAR(255),
    idempotency_key VARCHAR(255) UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT refund_payment_foreign_key
    FOREIGN KEY (payment_id)
    REFERENCES payments(id)
);

CREATE INDEX idx_refunds_status
ON refunds(status);

CREATE INDEX idx_refunds_payment_id
ON refunds(payment_id);

CREATE INDEX idx_refunds_provider_refund_id
ON refunds(provider_refund_id);