use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentProvider {
    Mock,
}
