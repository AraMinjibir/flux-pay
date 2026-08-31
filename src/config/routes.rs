use actix_web::web::{self};

use crate::api::payment_controller::{
    delete_payment, find_all_payments, find_payment_by_merchant, find_payment_by_method,
    find_payment_by_provider, find_payment_by_reference, find_payment_by_status, generate_payment,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1").service(
            web::scope("/payments")
                .route("", web::post().to(generate_payment))
                .route("", web::get().to(find_all_payments))
                .route("/delete/{id}", web::delete().to(delete_payment))
                .route(
                    "/reference/{reference}",
                    web::get().to(find_payment_by_reference),
                )
                .route("/status/{status}", web::get().to(find_payment_by_status))
                .route(
                    "/provider/{provider}",
                    web::get().to(find_payment_by_provider),
                )
                .route("/method/{method}", web::get().to(find_payment_by_method))
                .route(
                    "/merchant/{merchant_id}",
                    web::get().to(find_payment_by_merchant),
                ),
        ),
    );
}
