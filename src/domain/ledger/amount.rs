use crate::domain::{
    errors::ledger_error::LedgerError, shared::{currency::Currency, money::Money},
};

pub(crate) fn minor(money: &Money) -> i64 {
    money.amount()
}

pub(crate) fn currency_of(money: &Money) -> Currency {
    money.currency().clone()
}

pub(crate) fn money(minor: i64, currency: &Currency) -> Result<Money, LedgerError> {
    Money::new(minor, currency.clone()).map_err(|_| LedgerError::InvalidAmount)
}
