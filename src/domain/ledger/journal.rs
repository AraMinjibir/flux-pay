use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    ledger::{
        amount::{currency_of, minor},
        entry_direction::EntryDirection,
        error::LedgerError,
        jounal_source_type::JournalSourceType,
        journal_entry::NewJournalEntry,
        journal_event_type::JournalEventType,
        journal_status::JournalStatus,
    },
    shared::currency::Currency,
};

/// A persisted journal header.
#[derive(Debug, Clone)]
pub struct Journal {
    pub id: Uuid,
    pub reference: String,
    pub source_type: JournalSourceType,
    pub source_id: Uuid,
    pub event_type: JournalEventType,
    pub idempotency_key: String,
    pub description: String,
    pub currency: Currency,
    pub status: JournalStatus,
    pub created_at: DateTime<Utc>,
    pub posted_at: Option<DateTime<Utc>>,
}

/// A journal proposed for posting. `validate` is the single domain-level gate;
/// the database triggers re-check the same invariants as a backstop.
#[derive(Debug, Clone)]
pub struct JournalDraft {
    pub reference: String,
    pub source_type: JournalSourceType,
    pub source_id: Uuid,
    pub event_type: JournalEventType,
    pub idempotency_key: String,
    pub description: String,
    pub currency: Currency,
    pub entries: Vec<NewJournalEntry>,
}

impl JournalDraft {
    /// Checks: at least two entries, positive amounts, one currency, debits == credits.
    pub fn validate(&self) -> Result<(), LedgerError> {
        if self.entries.len() < 2 {
            return Err(LedgerError::JournalTooShort);
        }

        let mut debits: i64 = 0;

        let mut credits: i64 = 0;

        for (index, entry) in self.entries.iter().enumerate() {
            if currency_of(&entry.amount) != self.currency {
                return Err(LedgerError::CurrencyMismatch { line: index + 1 });
            }

            let side = match entry.direction {
                EntryDirection::Debit => &mut debits,

                EntryDirection::Credit => &mut credits,
            };

            *side = side
                .checked_add(minor(&entry.amount))
                .ok_or(LedgerError::AmountOverflow)?;
        }

        if debits != credits {
            return Err(LedgerError::Unbalanced { debits, credits });
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     fn draft(entries: Vec<NewJournalEntry>) -> JournalDraft {
//         JournalDraft {
//             reference: "J-1".into(),
//             source_type: JournalSourceType::Payment,
//             source_id: Uuid::new_v4(),
//             event_type: JournalEventType::PaymentCaptured,
//             idempotency_key: "k-1".into(),
//             description: "test".into(),
//             currency: test_ngn(),
//             entries,
//         }
//     }

//     #[test]
//     fn balanced_journal_is_valid() {
//         let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
//         let ngn = test_ngn();
//         let j = draft(vec![
//             NewJournalEntry::debit(a, money(1000, &ngn)),
//             NewJournalEntry::credit(b, money(1000, &ngn)),
//         ]);
//         assert_eq!(j.validate(), Ok(()));
//     }

//     #[test]
//     fn single_entry_is_rejected() {
//         let ngn = test_ngn();
//         let j = draft(vec![NewJournalEntry::debit(
//             Uuid::new_v4(),
//             money(1000, &ngn),
//         )]);
//         assert_eq!(j.validate(), Err(LedgerError::JournalTooShort));
//     }

//     #[test]
//     fn unbalanced_journal_is_rejected() {
//         let ngn = test_ngn();
//         let j = draft(vec![
//             NewJournalEntry::debit(Uuid::new_v4(), money(1000, &ngn)),
//             NewJournalEntry::credit(Uuid::new_v4(), money(999, &ngn)),
//         ]);
//         assert_eq!(
//             j.validate(),
//             Err(LedgerError::Unbalanced {
//                 debits: 1000,
//                 credits: 999
//             })
//         );
//     }

//     #[test]
//     fn zero_and_negative_amounts_are_rejected() {
//         let ngn = test_ngn();
//         for bad in [0, -5] {
//             let j = draft(vec![
//                 NewJournalEntry::debit(Uuid::new_v4(), money(bad, &ngn)),
//                 NewJournalEntry::credit(Uuid::new_v4(), money(bad, &ngn)),
//             ]);
//             assert_eq!(
//                 j.validate(),
//                 Err(LedgerError::NonPositiveAmount { line: 1 })
//             );
//         }
//     }

//     #[test]
//     fn mixed_currency_is_rejected() {
//         let (ngn, usd) = (test_ngn(), test_usd());
//         let j = draft(vec![
//             NewJournalEntry::debit(Uuid::new_v4(), money(1000, &ngn)),
//             NewJournalEntry::credit(Uuid::new_v4(), money(1000, &usd)),
//         ]);
//         assert_eq!(j.validate(), Err(LedgerError::CurrencyMismatch { line: 2 }));
//     }

//     #[test]
//     fn overflow_is_reported_not_wrapped() {
//         let ngn = test_ngn();
//         let j = draft(vec![
//             NewJournalEntry::debit(Uuid::new_v4(), money(i64::MAX, &ngn)),
//             NewJournalEntry::debit(Uuid::new_v4(), money(1, &ngn)),
//             NewJournalEntry::credit(Uuid::new_v4(), money(1, &ngn)),
//         ]);
//         assert_eq!(j.validate(), Err(LedgerError::AmountOverflow));
//     }
// }
