use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{ledger::journal_status::JournalStatus, payment::payment::Payment};
#[derive(Debug, Clone)]
pub struct Journal {
    pub id: Uuid,
    pub reference: String,
    pub description: String,
    pub source_type: Payment,
    pub status: JournalStatus,
    pub created_at: DateTime<Utc>,
}
