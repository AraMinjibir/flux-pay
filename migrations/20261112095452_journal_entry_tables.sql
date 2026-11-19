-- journal_entries: the debit/credit lines of a journal. Source of truth for balances.

CREATE TABLE journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    journal_id UUID NOT NULL REFERENCES journals(id),
    account_id UUID NOT NULL REFERENCES ledger_accounts(id),

    line_number SMALLINT NOT NULL,

    -- Positive minor currency units; direction carries the sign.
    amount BIGINT NOT NULL,
    direction VARCHAR(10) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT positive_entry_amount CHECK (amount > 0),
    CONSTRAINT positive_line_number CHECK (line_number > 0),
    CONSTRAINT valid_entry_direction CHECK (direction IN ('DEBIT', 'CREDIT')),
    CONSTRAINT unique_journal_line UNIQUE (journal_id, line_number)
);

CREATE INDEX idx_journal_entries_journal ON journal_entries (journal_id);
CREATE INDEX idx_journal_entries_account ON journal_entries (account_id);

-- Entries can only be added to (or removed from) a DRAFT journal, and only for
-- postable, active accounts in the journal's currency. Entries are never updated.
CREATE FUNCTION journal_entries_guard() RETURNS TRIGGER AS $$
DECLARE
    j_status   VARCHAR(20);
    j_currency CHAR(3);
    a_currency CHAR(3);
    a_postable BOOLEAN;
    a_active   BOOLEAN;
BEGIN
    IF TG_OP = 'UPDATE' THEN
        RAISE EXCEPTION 'journal entries are immutable; delete and re-insert while the journal is DRAFT';
    END IF;

    IF TG_OP = 'DELETE' THEN
        SELECT status INTO j_status FROM journals WHERE id = OLD.journal_id;
        IF j_status = 'POSTED' THEN
            RAISE EXCEPTION 'entries of posted journal % cannot be deleted', OLD.journal_id;
        END IF;
        RETURN OLD;
    END IF;

    -- INSERT. FOR SHARE serialises against a concurrent DRAFT -> POSTED update.
    SELECT status, currency INTO j_status, j_currency
    FROM journals WHERE id = NEW.journal_id FOR SHARE;

    IF j_status IS DISTINCT FROM 'DRAFT' THEN
        RAISE EXCEPTION 'cannot add entries to journal % (status %)', NEW.journal_id, j_status;
    END IF;

    SELECT currency, is_postable, is_active
    INTO a_currency, a_postable, a_active
    FROM ledger_accounts WHERE id = NEW.account_id;

    IF NOT a_postable THEN
        RAISE EXCEPTION 'account % is a control account and cannot be posted to', NEW.account_id;
    END IF;
    IF NOT a_active THEN
        RAISE EXCEPTION 'account % is inactive', NEW.account_id;
    END IF;
    IF a_currency <> j_currency THEN
        RAISE EXCEPTION 'account currency % does not match journal currency %', a_currency, j_currency;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_journal_entries_guard
    BEFORE INSERT OR UPDATE OR DELETE ON journal_entries
    FOR EACH ROW EXECUTE FUNCTION journal_entries_guard();