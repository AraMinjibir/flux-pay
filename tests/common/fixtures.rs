use flux_pay::domain::{
    payment::{method::PaymentMethod::Card, payment::Payment},
    shared::{currency::Currency::NGN, money::Money},
};
use uuid::Uuid;

pub fn test_payment() -> Payment {
    let merchant_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    Payment::generate_payment(
        merchant_id,
        Money::new(100000, NGN).expect("Amout should be positive"),
        Some("School fees".to_string()),
        Card,
    )
    .expect("Test payment should be valid")
}
