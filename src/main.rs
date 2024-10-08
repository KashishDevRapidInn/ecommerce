use ecommerce::config::configuration;
use ecommerce::db::establish_connection;
use ecommerce::startup::Application;
use ecommerce::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("ecommerce".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = configuration::Settings::new().expect("Failed to load configurations");
    let pool = establish_connection(&config.database.url).await;
    let port = 8080;

    let application = Application::build(port, pool, config.redis.uri).await?;
    application.run_until_stopped().await?;
    Ok(())
}
