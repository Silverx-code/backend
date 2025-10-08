use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use std::env;

pub async fn create_pool() -> Result<Pool, Box<dyn std::error::Error>> {
    // Fetch DATABASE_URL from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");

    // Parse DATABASE_URL into a tokio_postgres::Config
    let pg_config: tokio_postgres::Config = database_url.parse()?;

    // Create a Deadpool config and populate it from parsed config
    let mut cfg = Config::new();
    cfg.user = pg_config.get_user().map(|s| s.to_string());
    cfg.password = pg_config.get_password().map(|s| s.to_string());
    cfg.dbname = pg_config.get_dbname().map(|s| s.to_string());
    cfg.host = pg_config.get_hosts()
        .get(0)
        .map(|h| h.to_string());
    cfg.port = pg_config.get_ports().get(0).copied();

    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    // Create the pool
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    // Test the connection
    let client = pool.get().await?;
    client.query("SELECT 1", &[]).await?;

    println!("✅ Database connection pool established successfully");
    Ok(pool)
}