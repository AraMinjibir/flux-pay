use uuid::Uuid;

use crate::domain::{
    errors::ledger_error::LedgerError,
    ledger::{
        amount::{currency_of, minor, money},
        jounal_source_type::JournalSourceType,
        journal::JournalDraft,
        journal_entry::NewJournalEntry,
        journal_event_type::JournalEventType,
    },
    shared::money::Money,
};

pub struct PaymentCapturedInput {
    pub payment_id: Uuid,
    pub reference: String,
    pub idempotency_key: String,
    /// Gross amount the customer paid.
    pub gross: Money,
    /// FluxPay's fee, deducted from the merchant. May be zero.
    pub platform_fee: Money,
    /// The provider's receivable subledger account for the provider that processed it.
    pub provider_receivable: Uuid,
    /// The merchant's payable subledger account.
    pub merchant_payable: Uuid,
    /// Platform Fee Revenue (4000).
    pub platform_fee_revenue: Uuid,
}

/// The provider has confirmed the payment, so it now owes FluxPay the gross amount.
///
///   Dr Provider Receivable   gross
///     Cr Merchant Payable        gross - fee
///     Cr Platform Fee Revenue    fee   (omitted when zero)
pub fn payment_captured(input: PaymentCapturedInput) -> Result<JournalDraft, LedgerError> {
    let gross = minor(&input.gross);
    let fee = minor(&input.platform_fee);
    let currency = currency_of(&input.gross);

    if gross <= 0 {
        return Err(LedgerError::InvalidPostingInput("gross must be positive"));
    }
    if fee < 0 {
        return Err(LedgerError::InvalidPostingInput("fee cannot be negative"));
    }
    if currency_of(&input.platform_fee) != currency {
        return Err(LedgerError::InvalidPostingInput(
            "fee currency differs from gross",
        ));
    }
    if fee > gross {
        return Err(LedgerError::FeeExceedsGross);
    }

    let net = gross - fee;
    let mut entries = vec![NewJournalEntry::debit(
        input.provider_receivable,
        input.gross.clone(),
    )];
    if net > 0 {
        entries.push(NewJournalEntry::credit(
            input.merchant_payable,
            money(net, &currency)?,
        ));
    }
    if fee > 0 {
        entries.push(NewJournalEntry::credit(
            input.platform_fee_revenue,
            money(fee, &currency)?,
        ));
    }

    Ok(JournalDraft {
        reference: input.reference,
        source_type: JournalSourceType::Payment,
        source_id: input.payment_id,
        event_type: JournalEventType::PaymentCaptured,
        idempotency_key: input.idempotency_key,
        description: "Payment captured".into(),
        currency,
        entries,
    })
}

pub struct ProviderSettledInput {
    pub settlement_id: Uuid,
    pub reference: String,
    pub idempotency_key: String,
    /// Cash that actually landed in FluxPay's bank account.
    pub received: Money,
    /// Fee the provider withheld from the settlement. May be zero.
    pub provider_fee: Money,
    /// The provider's receivable subledger account.
    pub provider_receivable: Uuid,
    /// Settlement Bank (1000).
    pub settlement_bank: Uuid,
    /// Provider Fee Expense (5000).
    pub provider_fee_expense: Uuid,
}

/// The provider has paid out. It clears the receivable for what it owed
/// (received + provider fee), and any withheld fee becomes an expense.
///
///   Dr Settlement Bank         received
///   Dr Provider Fee Expense    provider_fee   (omitted when zero)
///     Cr Provider Receivable       received + provider_fee
pub fn provider_settled(input: ProviderSettledInput) -> Result<JournalDraft, LedgerError> {
    let received = minor(&input.received);
    let fee = minor(&input.provider_fee);
    let currency = currency_of(&input.received);

    if received <= 0 {
        return Err(LedgerError::InvalidPostingInput(
            "received must be positive",
        ));
    }
    if fee < 0 {
        return Err(LedgerError::InvalidPostingInput(
            "provider fee cannot be negative",
        ));
    }
    if currency_of(&input.provider_fee) != currency {
        return Err(LedgerError::InvalidPostingInput(
            "provider fee currency differs from received",
        ));
    }

    let cleared = received
        .checked_add(fee)
        .ok_or(LedgerError::AmountOverflow)?;

    let mut entries = vec![NewJournalEntry::debit(
        input.settlement_bank,
        input.received.clone(),
    )];
    if fee > 0 {
        entries.push(NewJournalEntry::debit(
            input.provider_fee_expense,
            money(fee, &currency)?,
        ));
    }
    entries.push(NewJournalEntry::credit(
        input.provider_receivable,
        money(cleared, &currency)?,
    ));

    Ok(JournalDraft {
        reference: input.reference,
        source_type: JournalSourceType::Settlement,
        source_id: input.settlement_id,
        event_type: JournalEventType::ProviderSettled,
        idempotency_key: input.idempotency_key,
        description: "Provider settlement received".into(),
        currency,
        entries,
    })
}
