use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use std::{env, error::Error};

pub async fn create_pool() -> Result<Pool, Box<dyn Error>> {
    // Fetch DATABASE_URL from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");

    // Parse the DATABASE_URL into a Postgres config
    let pg_config: tokio_postgres::Config = database_url.parse()?;

    // Create a manager for the connection pool
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);

    // Build the connection pool
    let pool = Pool::builder(mgr)
        .max_size(16)
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap();

    // Test the connection
    let client = pool.get().await?;
    client.query("SELECT 1", &[]).await?;

    println!("✅ Database connection pool established successfully");
    Ok(pool)
}
