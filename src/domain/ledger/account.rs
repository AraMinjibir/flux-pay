use uuid::Uuid;

use crate::domain::{
    ledger::{account_type::AccountType, entry_direction::EntryDirection, owner_type::OwnerType},
    shared::currency::Currency,
};

#[derive(Debug, Clone)]
pub struct LedgerAccount {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub currency: Currency,
    pub owner_type: OwnerType,
    pub owner_id: Option<Uuid>,
    /// Set on merchant/provider subledger accounts; points at the GL control account.
    pub parent_account_id: Option<Uuid>,
    /// Control accounts are not postable.
    pub is_postable: bool,
    pub is_active: bool,
}

impl LedgerAccount {
    pub fn normal_balance(&self) -> EntryDirection {
        self.account_type.normal_balance()
    }
}