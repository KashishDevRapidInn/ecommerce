use diesel::prelude::*;
use diesel::sql_types::Json;
use diesel_async_migrations::{embed_migrations, EmbeddedMigrations};
pub const MIGRATIONS: EmbeddedMigrations = diesel_async_migrations::embed_migrations!("migrations");

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use diesel_async::pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager};
use diesel_async::RunQueryDsl;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use ecommerce::config::configuration;
use ecommerce::db::create_database;
use ecommerce::db::PgPool;
use ecommerce::schema::admins::{self, dsl as admin_dsl};
use ecommerce::schema::customers::{self, dsl as customer_dsl};
use ecommerce::schema::products::{self, dsl as product_dsl};
use ecommerce::startup::Application;
use ecommerce::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use serde_json::Value;
use tokio;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // if std::env::var("TEST_LOG").is_ok() {
    //     let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
    //     init_subscriber(subscriber);
    // } else {
    let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
    init_subscriber(subscriber);
    // };
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
        let mut conn = pool
            .get()
            .await
            .expect("Failed to get db connection from pool");

        diesel::insert_into(customer_dsl::customers)
            .values((
                customer_dsl::id.eq(self.user_id),
                customer_dsl::username.eq(self.username.clone()),
                customer_dsl::password_hash.eq(hashed_password.clone()),
                customer_dsl::email.eq(self.user_email.clone()),
            ))
            .execute(&mut conn)
            .await
            .expect("Failed to create test customers.");

        diesel::insert_into(admin_dsl::admins)
            .values((
                admin_dsl::id.eq(self.user_id),
                admin_dsl::username.eq(self.username.clone()),
                admin_dsl::password_hash.eq(hashed_password.clone()),
            ))
            .execute(&mut conn)
            .await
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
    pub test_db_url: String,
}
impl TestApp {
    pub async fn login_customer(&self, body: Value) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/login", &self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute login customer request")
    }

    pub async fn update_customer(&self, body: Value, token: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/protected/update", &self.address))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .expect("Failed to execute update customer request")
    }

    pub async fn view_customer(&self, token: String) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/protected/view", &self.address))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to execute view customer request")
    }

    pub async fn create_order(&self, body: Value, token: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/protected/orders/new", &self.address))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .expect("Failed to execute create order request")
    }

    pub async fn get_order(&self, order_id: &str, token: String) -> reqwest::Response {
        self.api_client
            .get(&format!(
                "{}/protected/orders/{}/view",
                &self.address, &order_id
            ))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to execute get order request")
    }

    pub async fn get_all_orders(&self, token: String) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/protected/orders/list/all", &self.address))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to execute get all orders by a customer request")
    }

    pub async fn login_admin(&self, body: Value) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/admin/login", &self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute login admin request")
    }

    pub async fn update_order_status(&self, body: Value, token: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/protected/admin/update_status", &self.address))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .expect("Failed to execute update status request by admin")
    }

    pub async fn logout_customer(&self, token: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/protected/logout", &self.address))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to execute logout customer request")
    }

    pub async fn logout_admin(&self, token: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/protected/admin/logout", &self.address))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to execute logout customer request")
    }
}

pub async fn run_migrations(url: impl AsRef<str>) -> Result<(), std::io::Error> {
    // Establish a connection
    let mut conn = AsyncPgConnection::establish(url.as_ref())
        .await
        .expect("Failed to run migrations");

    // Run pending migrations
    MIGRATIONS
        .run_pending_migrations(&mut conn)
        .await
        .expect("Failed to run migrations");

    Ok(())
}
pub async fn spawn_app() -> TestApp {
    // To Ensure that the tracing stack is only initialized once
    Lazy::force(&TRACING);

    let database_name = Uuid::new_v4().to_string();
    let config = configuration::Settings::new().expect("Failed to load configurations");
    create_database(&database_name, config.database.test_url.clone()).await;

    let new_database_url = format!("{}/{}", config.database.test_url, database_name);

    //building pool
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        new_database_url.clone(),
    );
    let pool = Pool::builder(manager)
        .max_size(16)
        .build()
        .expect("Failed to create pool");
    // Run migrations
    // let mut conn = pool
    //     .get()
    //     .await
    //     .expect("Couldn't get db connection from Pool");

    if let Err(err) = run_migrations(&new_database_url).await {
        eprintln!("Error running migrations: {}", err);
    }

    let application = Application::build(0, pool.clone(), config.redis.uri)
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
        test_db_url: config.database.test_url,
    };
    testapp.test_user.store(&testapp.db_pool).await;
    testapp
}

pub async fn seed_products(pool: PgPool) -> Result<(), diesel::result::Error> {
    let data = vec![(
        Uuid::parse_str("5fcd7d83-7adf-4d4d-931a-68b9678009db").unwrap(),
        "Laptop".to_string(),
        true,
        50000,
    )];
    let mut conn = pool
        .get()
        .await
        .expect("Failed to get db connection from Pool");
    for (id, name, is_available, price) in data {
        diesel::insert_into(product_dsl::products)
            .values((
                product_dsl::id.eq(id),
                product_dsl::name.eq(name),
                product_dsl::is_available.eq(is_available),
                product_dsl::price.eq(price),
            ))
            .execute(&mut conn)
            .await?;
    }

    println!("successfully added products");
    Ok(())
}
