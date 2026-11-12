use uuid::Uuid;

use crate::domain::{ledger::account_type::AccountType, shared::currency::Currency};

#[derive(Debug, Clone)]
pub struct LedgerAccount {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub currency: Currency,
    pub owner_id: Option<Uuid>,
    pub is_active: bool,
}
