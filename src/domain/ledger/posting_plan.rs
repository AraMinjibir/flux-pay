use std::collections::BTreeMap;

use uuid::Uuid;

use crate::domain::{
    errors::ledger_error::LedgerError,
    ledger::{amount::minor, entry_direction::EntryDirection, journal::JournalDraft},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostingPlan {
    pub entry_account_ids: Vec<Uuid>,
    pub entry_line_numbers: Vec<i16>,
    pub entry_amounts: Vec<i64>,
    pub entry_directions: Vec<&'static str>,

    pub balance_account_ids: Vec<Uuid>,
    pub balance_debits: Vec<i64>,
    pub balance_credits: Vec<i64>,
}

impl PostingPlan {
    /// Runs `draft.validate()` first, so a plan only exists for a balanced journal.
    pub fn from_draft(draft: &JournalDraft) -> Result<Self, LedgerError> {
        draft.validate()?;

        let mut plan = Self {
            entry_account_ids: Vec::with_capacity(draft.entries.len()),
            entry_line_numbers: Vec::with_capacity(draft.entries.len()),
            entry_amounts: Vec::with_capacity(draft.entries.len()),
            entry_directions: Vec::with_capacity(draft.entries.len()),
            balance_account_ids: Vec::new(),
            balance_debits: Vec::new(),
            balance_credits: Vec::new(),
        };

        // (debits, credits) per account; BTreeMap gives the sorted, unique ordering.
        let mut deltas: BTreeMap<Uuid, (i64, i64)> = BTreeMap::new();

        for (index, entry) in draft.entries.iter().enumerate() {
            let amount = minor(&entry.amount);
            let line = i16::try_from(index + 1)
                .map_err(|_| LedgerError::InvalidPostingInput("too many journal entries"))?;

            plan.entry_account_ids.push(entry.account_id);
            plan.entry_line_numbers.push(line);
            plan.entry_amounts.push(amount);
            plan.entry_directions.push(entry.direction.as_str());

            let delta = deltas.entry(entry.account_id).or_insert((0, 0));
            let side = match entry.direction {
                EntryDirection::Debit => &mut delta.0,
                EntryDirection::Credit => &mut delta.1,
            };
            *side = side
                .checked_add(amount)
                .ok_or(LedgerError::AmountOverflow)?;
        }

        for (account_id, (debits, credits)) in deltas {
            plan.balance_account_ids.push(account_id);
            plan.balance_debits.push(debits);
            plan.balance_credits.push(credits);
        }

        Ok(plan)
    }
}
