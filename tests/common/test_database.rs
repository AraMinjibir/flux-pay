use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use std::sync::LazyLock;
use tokio::sync::OnceCell;

pub struct TestDb {
    pub pool: PgPool,
}

static INIT: LazyLock<OnceCell<()>> = LazyLock::new(OnceCell::const_new);

impl TestDb {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();

        let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@postgres:5432/doodoo_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .expect("Unable to connect to the db");

        Self { pool }
    }

    pub async fn init(pool: &PgPool) {
        INIT.get_or_init(|| async {
            sqlx::migrate!()
                .run(pool)
                .await
                .expect("Unable to run migration");
        })
        .await;
    }

    pub async fn reset(&self) {
        sqlx::query!(
            r#"
            TRUNCATE TABLE
                payments
            RESTART IDENTITY CASCADE;    
            "#
        )
        .execute(&self.pool)
        .await
        .expect("Unable to reset bd");
    }
}
