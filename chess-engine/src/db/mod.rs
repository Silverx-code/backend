use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use std::env;

pub async fn create_pool() -> Result<Pool, Box<dyn std::error::Error>> {
    // Fetch DATABASE_URL from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");

    // Create manager directly from connection string
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::new_from_stringlike(database_url, NoTls, mgr_config)?;
    let pool = Pool::builder(mgr)
        .max_size(16)
        .runtime(Runtime::Tokio1)
        .build()?;

    // Test connection
    let client = pool.get().await?;
    client.query("SELECT 1", &[]).await?;

    Ok(pool)
}