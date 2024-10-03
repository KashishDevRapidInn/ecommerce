use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::MigrationHarness;
use dotenv::dotenv;
use ecommerce::db::create_database;
use ecommerce::db::PgPool;
use ecommerce::schema::admins::{self, dsl as admin_dsl};
use ecommerce::schema::customers::{self, dsl as customer_dsl};
use ecommerce::startup::Application;
use ecommerce::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use std::env;
use tokio;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    dotenv().ok();
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the value of `TEST_LOG`
    // because the sink is part of the type returned by `get_subscriber`, therefore they are not the
    // same type. We could work around it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
    pub user_email: String,
}
impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
            user_email: "kk@gmail.com".to_string(),
        }
    }
    async fn store(&self, pool: &PgPool) {
        let salt_argon = SaltString::generate(&mut rand::thread_rng());
        let hashed_password = Argon2::default()
            .hash_password(self.password.as_bytes(), &salt_argon)
            .unwrap()
            .to_string();
        // dbg!(&hashed_password);
        let mut conn = pool.get().expect("Failed to get db connection from pool");

        diesel::insert_into(customer_dsl::customers)
            .values((
                customer_dsl::id.eq(self.user_id),
                customer_dsl::username.eq(self.username.clone()),
                customer_dsl::password_hash.eq(hashed_password.clone()),
                customer_dsl::email.eq(self.user_email.clone()),
            ))
            .execute(&mut conn)
            .expect("Failed to create test customers.");

        diesel::insert_into(admin_dsl::admins)
            .values((
                admin_dsl::id.eq(self.user_id),
                admin_dsl::username.eq(self.username.clone()),
                admin_dsl::password_hash.eq(hashed_password.clone()),
            ))
            .execute(&mut conn)
            .expect("Failed to create test admin.");
    }
}

pub struct TestApp {
    pub port: u16,
    pub address: String,
    pub db_pool: PgPool,
    pub database_name: String,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
}
pub fn run_db_migrations(conn: &mut impl MigrationHarness<diesel::pg::Pg>) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations");
}

pub async fn spawn_app() -> TestApp {
    // To Ensure that the tracing stack is only initialized once
    Lazy::force(&TRACING);

    dotenv().ok();
    let database_name = Uuid::new_v4().to_string();
    let database_url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL must be set");
    create_database(&database_name);

    let new_database_url = format!("{}/{}", database_url, database_name);
    let manager = ConnectionManager::<PgConnection>::new(new_database_url.clone());
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    // Run migrations
    let mut conn = pool.get().expect("Couldn't get db connection from Pool");
    run_db_migrations(&mut conn);

    let application = Application::build(0, pool.clone())
        .await
        .expect("Failed to build application");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application_port);
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    let testapp = TestApp {
        port: application_port,
        address,
        db_pool: pool.clone(),
        database_name,
        test_user: TestUser::generate(),
        api_client: client,
    };
    testapp.test_user.store(&testapp.db_pool).await;
    testapp
}
