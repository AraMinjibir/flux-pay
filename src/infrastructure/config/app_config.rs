use std::time::Duration;

pub struct AppConfig {
    pub mock: MockConfig,
    pub paystack: PaystackConfig,
    pub stripe: StripeConfig,
    pub interswitch: InterswitchConfig,
    pub zainpay: ZainpayConfig,
}

pub struct MockConfig {
    pub base_url: String,
}

pub struct InterswitchConfig {
    pub secret_key: String,
    pub base_url: String,
    pub client_id: String,
}
pub struct StripeConfig {
    pub secret_key: String,
    pub base_url: String,
}

pub struct PaystackConfig {
    pub secret_key: String,
    pub base_url: String,
}

pub struct ZainpayConfig {
    pub secret_key: String,
    pub base_url: String,
}


#[derive(Debug, Clone)]
pub struct IdempotencyConfig {
    pub ttl: Duration,
}


impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(24 * 60 * 60),
        }
    }
}