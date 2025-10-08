use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use std::env;

pub async fn create_pool() -> Result<Pool, Box<dyn std::error::Error>> {
    // Fetch DATABASE_URL from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");

    // Parse connection string into a Config
    let mut cfg = Config::new();
    cfg.pg_config = database_url.parse()?;
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    // Create connection pool
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    // Test connection
    let client = pool.get().await?;
    client.query("SELECT 1", &[]).await?;

    println!("✅ Database connection pool established successfully");
    Ok(pool)
}