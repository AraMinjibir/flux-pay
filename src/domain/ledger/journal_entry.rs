use uuid::Uuid;

use crate::domain::{ledger::entry_direction::EntryDirection, shared::money::Money};

/// A persisted journal line.
#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub id: Uuid,
    pub journal_id: Uuid,
    pub account_id: Uuid,
    pub line_number: i16,
    pub amount: Money,
    pub direction: EntryDirection,
}

/// A line proposed for a journal that has not been persisted yet.
#[derive(Debug, Clone)]
pub struct NewJournalEntry {
    pub account_id: Uuid,
    pub amount: Money,
    pub direction: EntryDirection,
}

impl NewJournalEntry {
    pub fn debit(account_id: Uuid, amount: Money) -> Self {
        Self {
            account_id,
            amount,
            direction: EntryDirection::Debit,
        }
    }

    pub fn credit(account_id: Uuid, amount: Money) -> Self {
        Self {
            account_id,
            amount,
            direction: EntryDirection::Credit,
        }
    }
}
