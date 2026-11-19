-- journals: one row per accounting event. DRAFT while entries are being added,
-- immutable once POSTED.

CREATE TABLE journals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    reference VARCHAR(100) NOT NULL UNIQUE,

    -- PAYMENT, REFUND, PAYOUT, SETTLEMENT, etc.
    source_type VARCHAR(30) NOT NULL,
    source_id UUID NOT NULL,

    -- The precise accounting event, e.g. PAYMENT_CAPTURED, PROVIDER_SETTLED.
    event_type VARCHAR(50) NOT NULL,

    idempotency_key VARCHAR(150) NOT NULL UNIQUE,

    description TEXT NOT NULL,
    currency CHAR(3) NOT NULL,

    status VARCHAR(20) NOT NULL DEFAULT 'DRAFT',

    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    posted_at TIMESTAMPTZ,

    -- One accounting event of a given type per source record. Also serves
    -- lookups by (source_type, source_id) via its leading columns.
    CONSTRAINT unique_journal_source_event UNIQUE (source_type, source_id, event_type),

    CONSTRAINT valid_journal_status
        CHECK (status IN ('DRAFT', 'POSTED')),

    CONSTRAINT valid_journal_currency
        CHECK (currency ~ '^[A-Z]{3}$'),

    CONSTRAINT valid_posted_timestamp CHECK (
        (status = 'DRAFT' AND posted_at IS NULL)
        OR
        (status = 'POSTED' AND posted_at IS NOT NULL)
    )
);

CREATE INDEX idx_journals_event_type ON journals (event_type);

-- Immutability once POSTED, and the debits == credits check on posting.
-- Reads journal_entries at execution time, so it does not need that table to exist yet.
CREATE FUNCTION journals_guard() RETURNS TRIGGER AS $$
DECLARE
    total_debits  NUMERIC;
    total_credits NUMERIC;
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status = 'POSTED' THEN
            RAISE EXCEPTION 'posted journal % cannot be deleted', OLD.id;
        END IF;
        RETURN OLD;
    END IF;

    -- UPDATE
    IF OLD.status = 'POSTED' THEN
        RAISE EXCEPTION 'posted journal % is immutable', OLD.id;
    END IF;

    IF NEW.id <> OLD.id
       OR NEW.reference <> OLD.reference
       OR NEW.idempotency_key <> OLD.idempotency_key
       OR NEW.source_type <> OLD.source_type
       OR NEW.source_id <> OLD.source_id
       OR NEW.event_type <> OLD.event_type
       OR NEW.currency <> OLD.currency
    THEN
        RAISE EXCEPTION 'journal %: identity fields are immutable', OLD.id;
    END IF;

    IF NEW.status = 'POSTED' THEN
        SELECT
            COALESCE(SUM(amount) FILTER (WHERE direction = 'DEBIT'), 0),
            COALESCE(SUM(amount) FILTER (WHERE direction = 'CREDIT'), 0)
        INTO total_debits, total_credits
        FROM journal_entries
        WHERE journal_id = OLD.id;

        IF total_debits = 0 OR total_debits <> total_credits THEN
            RAISE EXCEPTION 'journal % is not balanced (debits %, credits %)',
                OLD.id, total_debits, total_credits;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_journals_guard
    BEFORE UPDATE OR DELETE ON journals
    FOR EACH ROW EXECUTE FUNCTION journals_guard();