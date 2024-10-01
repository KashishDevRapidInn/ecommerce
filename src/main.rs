use actix_web::{web, App, HttpServer};
pub mod Errors;
mod db;
mod db_models;
pub mod routes;
pub mod schema;
pub mod session_state;
pub mod telemetry;
pub mod validations;
// use crate::routes::products::seed::seed_products;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use db::establish_connection;
use dotenv::dotenv;
use routes::{
    admin::admin::{login_admin, register_admin, update_status},
    customer::customer::{
        login_customer, logout_customer, register_customer, update_customer, view_customer,
    },
    order::order::{create_order, get_order, list_orders},
};
use telemetry::{get_subscriber, init_subscriber};
use tracing_actix_web::TracingLogger;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("ecommerce".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    dotenv().ok();

    let redis_uri = std::env::var("REDIS_URI").expect("Failed to get redis uri");
    let redis_store = RedisSessionStore::new(redis_uri).await.map_err(|e| {
        eprintln!("Failed to create Redis session store: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Redis connection failed")
    })?;
    let secret_key = Key::generate();
    let pool = establish_connection();
    // seed_products(pool.clone());
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .app_data(web::Data::new(pool.clone()))
            .route("register", web::post().to(register_customer))
            .route("login", web::post().to(login_customer))
            .route("logout", web::post().to(logout_customer))
            .route("update", web::post().to(update_customer))
            .route("view", web::post().to(view_customer))
            .route("/orders/new", web::post().to(create_order))
            .route("/orders/{id}/view", web::get().to(get_order))
            .route("/orders/list/all", web::get().to(list_orders))
            .route("/admin/register", web::post().to(register_admin))
            .route("/admin/login", web::post().to(login_admin))
            .route("/admin/update_status", web::post().to(update_status))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
