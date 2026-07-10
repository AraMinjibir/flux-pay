pub struct AppConfig {
    pub mock: MockConfig,
    pub paystack: PaystackConfig,
    pub stripe: StripeConfig,
    pub betastack: InterswitchConfig,
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
