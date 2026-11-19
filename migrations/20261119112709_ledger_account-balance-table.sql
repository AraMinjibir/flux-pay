-- ledger_account_balances: projection of journal_entries, one row per account.
-- Updated only by the posting transaction; always re-derivable from journal_entries.

CREATE TABLE ledger_account_balances (
    account_id UUID PRIMARY KEY REFERENCES ledger_accounts(id),

    -- Running totals in minor units. The signed balance is derived from the
    -- account's normal_balance.
    total_debits BIGINT NOT NULL DEFAULT 0,
    total_credits BIGINT NOT NULL DEFAULT 0,

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT non_negative_debits CHECK (total_debits >= 0),
    CONSTRAINT non_negative_credits CHECK (total_credits >= 0)
);

CREATE TRIGGER trg_ledger_account_balances_updated_at
    BEFORE UPDATE ON ledger_account_balances
    FOR EACH ROW EXECUTE FUNCTION ledger_set_updated_at();

-- Every account gets a zeroed balance row, so posting can always UPDATE it.
-- Lives here because it writes to this table, but fires on ledger_accounts.
CREATE FUNCTION ledger_accounts_create_balance() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO ledger_account_balances (account_id) VALUES (NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_ledger_accounts_create_balance
    AFTER INSERT ON ledger_accounts
    FOR EACH ROW EXECUTE FUNCTION ledger_accounts_create_balance();