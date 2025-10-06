use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use std::env;

pub async fn create_pool() -> Result<Pool, Box<dyn std::error::Error>> {
    // Get database configuration from environment variables
    let mut cfg = Config::new();
    
    cfg.host = Some(env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string()));
    cfg.port = Some(
        env::var("DATABASE_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse()
            .unwrap_or(5432),
    );
    cfg.dbname = Some(env::var("DATABASE_NAME").unwrap_or_else(|_| "chess_app".to_string()));
    cfg.user = Some(env::var("DATABASE_USER").unwrap_or_else(|_| "postgres".to_string()));
    cfg.password = Some(env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "postgres".to_string()));
    
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    
    // Test the connection
    let client = pool.get().await?;
    client.query("SELECT 1", &[]).await?;
    
    Ok(pool)
}