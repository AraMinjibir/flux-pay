use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Display, EnumString, PartialEq, Eq, Hash)]
pub enum Currency {
    NGN,
    USD,
    EUR,
    GBP,
    AED,
    SAR,
}
