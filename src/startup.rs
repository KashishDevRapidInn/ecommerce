use crate::db::PgPool;
use crate::middleware::jwt_auth_middleware;
use crate::routes::{
    admin::admin::{login_admin, logout_admin, register_admin, update_status},
    customer::customer::{
        login_customer, logout_customer, register_customer, update_customer, view_customer,
    },
    health_check::health_check,
    order::order::{create_order, get_order, list_orders},
};
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::{dev::Server, web, App, HttpServer};
use actix_web_lab::middleware::from_fn;
use std::env;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

/******************************************/
// Initializing Redis connection
/******************************************/
pub async fn init_redis() -> Result<RedisSessionStore, std::io::Error> {
    let redis_uri = env::var("REDIS_URI").expect("Failed to get redis uri");
    RedisSessionStore::new(redis_uri).await.map_err(|e| {
        eprintln!("Failed to create Redis session store: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Redis connection failed")
    })
}

pub fn generate_secret_key() -> Key {
    Key::generate()
}
/**************************************************************/
// Application State re reuse the same code in main and tests
/***************************************************************/
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(port: u16, pool: PgPool) -> Result<Self, std::io::Error> {
        let listener = if port == 0 {
            TcpListener::bind("127.0.0.1:0")?
        } else {
            let address = format!("127.0.0.1:{}", port);
            TcpListener::bind(&address)?
        };

        let actual_port = listener.local_addr()?.port();

        let server = run_server(listener, pool.clone()).await?;
        Ok(Self {
            port: actual_port,
            server,
        })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

/******************************************/
// Running Server
/******************************************/
pub async fn run_server(listener: TcpListener, pool: PgPool) -> Result<Server, std::io::Error> {
    let redis_store = init_redis().await?;
    let secret_key = generate_secret_key();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .app_data(web::Data::new(pool.clone()))
            .route("/register", web::post().to(register_customer))
            .route("/login", web::post().to(login_customer))
            .route("/admin/register", web::post().to(register_admin))
            .route("/admin/login", web::post().to(login_admin))
            .route("/health_check", web::get().to(health_check))
            .service(
                web::scope("/protected")
                    .wrap(from_fn(jwt_auth_middleware))
                    .route("/logout", web::post().to(logout_customer))
                    .route("/update", web::post().to(update_customer))
                    .route("/view", web::get().to(view_customer))
                    .route("/orders/new", web::post().to(create_order))
                    .route("/orders/{id}/view", web::get().to(get_order))
                    .route("/orders/list/all", web::get().to(list_orders))
                    .route("/admin/update_status", web::post().to(update_status))
                    .route("/admin/logout", web::post().to(logout_admin)),
            )
    })
    .listen(listener)?
    .run();
    Ok(server)
}
