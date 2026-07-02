use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Display, EnumString)]
pub enum Currency {
    NGN,
    USD,
    EUR,
    GBP,
    AED,
    SAR,
}
