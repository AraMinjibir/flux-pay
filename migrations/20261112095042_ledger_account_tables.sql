-- ledger_accounts: chart of accounts and merchant/provider subledger accounts.
--
-- Merchants are rows with UUIDs; providers are a code-level enum, so provider
-- accounts carry no owner_id and are identified by code (e.g. 1100-PAYSTACK).

CREATE TABLE ledger_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    code VARCHAR(50) NOT NULL,
    name VARCHAR(120) NOT NULL,

    account_type VARCHAR(20) NOT NULL,
    normal_balance VARCHAR(10) NOT NULL,

    currency CHAR(3) NOT NULL,

    -- PLATFORM, MERCHANT, or PROVIDER
    owner_type VARCHAR(20) NOT NULL,
    owner_id UUID,

    -- A merchant/provider subledger account belongs to a GL control account.
    parent_account_id UUID REFERENCES ledger_accounts(id),

    -- Control accounts are not postable; only leaf accounts receive entries.
    is_postable BOOLEAN NOT NULL DEFAULT TRUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_account_code_currency UNIQUE (code, currency),

    CONSTRAINT valid_account_type
        CHECK (account_type IN ('ASSET', 'LIABILITY', 'EQUITY', 'REVENUE', 'EXPENSE')),

    CONSTRAINT valid_normal_balance
        CHECK (normal_balance IN ('DEBIT', 'CREDIT')),

    CONSTRAINT normal_balance_matches_type CHECK (
        (account_type IN ('ASSET', 'EXPENSE') AND normal_balance = 'DEBIT')
        OR
        (account_type IN ('LIABILITY', 'EQUITY', 'REVENUE') AND normal_balance = 'CREDIT')
    ),

    CONSTRAINT valid_owner_type
        CHECK (owner_type IN ('PLATFORM', 'MERCHANT', 'PROVIDER')),

    CONSTRAINT valid_owner_reference CHECK (
        (owner_type IN ('PLATFORM', 'PROVIDER') AND owner_id IS NULL)
        OR
        (owner_type = 'MERCHANT' AND owner_id IS NOT NULL)
    ),

    CONSTRAINT valid_currency
        CHECK (currency ~ '^[A-Z]{3}$'),

    CONSTRAINT no_self_parent
        CHECK (parent_account_id IS NULL OR parent_account_id <> id)
);

CREATE INDEX idx_ledger_accounts_parent ON ledger_accounts (parent_account_id);
CREATE INDEX idx_ledger_accounts_owner ON ledger_accounts (owner_type, owner_id);

-- At most one subledger account per owner under a given control account.
CREATE UNIQUE INDEX uq_ledger_accounts_subledger_owner
    ON ledger_accounts (parent_account_id, owner_id)
    WHERE parent_account_id IS NOT NULL;

-- Shared by ledger_accounts and ledger_account_balances.
CREATE FUNCTION ledger_set_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION ledger_accounts_guard() RETURNS TRIGGER AS $$
DECLARE
    parent ledger_accounts%ROWTYPE;
BEGIN
    IF TG_OP = 'UPDATE' THEN
        IF NEW.code <> OLD.code
           OR NEW.account_type <> OLD.account_type
           OR NEW.normal_balance <> OLD.normal_balance
           OR NEW.currency <> OLD.currency
           OR NEW.owner_type <> OLD.owner_type
           OR NEW.owner_id IS DISTINCT FROM OLD.owner_id
           OR NEW.parent_account_id IS DISTINCT FROM OLD.parent_account_id
        THEN
            RAISE EXCEPTION 'ledger account %: identity fields are immutable', OLD.id;
        END IF;

        IF NEW.is_postable AND NOT OLD.is_postable
           AND EXISTS (SELECT 1 FROM ledger_accounts WHERE parent_account_id = OLD.id)
        THEN
            RAISE EXCEPTION 'ledger account %: control account with children cannot become postable', OLD.id;
        END IF;

        RETURN NEW;
    END IF;

    -- INSERT: a subledger account must hang under a compatible control account.
    IF NEW.parent_account_id IS NOT NULL THEN
        SELECT * INTO parent FROM ledger_accounts WHERE id = NEW.parent_account_id;

        IF FOUND THEN
            IF parent.is_postable THEN
                RAISE EXCEPTION 'parent account % must be a non-postable control account', parent.id;
            END IF;
            IF parent.account_type <> NEW.account_type
               OR parent.normal_balance <> NEW.normal_balance
               OR parent.currency <> NEW.currency
            THEN
                RAISE EXCEPTION 'account % must match parent % on type, normal balance and currency',
                    NEW.code, parent.code;
            END IF;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_ledger_accounts_guard
    BEFORE INSERT OR UPDATE ON ledger_accounts
    FOR EACH ROW EXECUTE FUNCTION ledger_accounts_guard();

CREATE TRIGGER trg_ledger_accounts_updated_at
    BEFORE UPDATE ON ledger_accounts
    FOR EACH ROW EXECUTE FUNCTION ledger_set_updated_at();