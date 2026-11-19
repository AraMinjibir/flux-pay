INSERT INTO
    ledger_accounts (
        code,
        name,
        account_type,
        normal_balance,
        currency,
        owner_type,
        is_postable
    )
VALUES
    (
        '1000',
        'Settlement Bank',
        'ASSET',
        'DEBIT',
        'NGN',
        'PLATFORM',
        TRUE
    ),
    (
        '1100',
        'Provider Receivable',
        'ASSET',
        'DEBIT',
        'NGN',
        'PLATFORM',
        FALSE
    ),
    (
        '1900',
        'Reconciliation Suspense',
        'ASSET',
        'DEBIT',
        'NGN',
        'PLATFORM',
        TRUE
    ),
    (
        '2000',
        'Merchant Payable',
        'LIABILITY',
        'CREDIT',
        'NGN',
        'PLATFORM',
        FALSE
    ),
    (
        '4000',
        'Platform Fee Revenue',
        'REVENUE',
        'CREDIT',
        'NGN',
        'PLATFORM',
        TRUE
    ),
    (
        '5000',
        'Provider Fee Expense',
        'EXPENSE',
        'DEBIT',
        'NGN',
        'PLATFORM',
        TRUE
    ) ON CONFLICT (code, currency) DO NOTHING;