use uuid::Uuid;

use crate::domain::{ledger::entry_direction::EntryDirection, shared::money::Money};
#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub id: Uuid,
    pub journal_id: Uuid,
    pub account_id: Uuid,
    pub amount: Money,
    pub direction: EntryDirection,
}
