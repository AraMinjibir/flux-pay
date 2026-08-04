use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Display, EnumString, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Currency {
    NGN,
    USD,
    EUR,
    GBP,
    AED,
    SAR,
}
