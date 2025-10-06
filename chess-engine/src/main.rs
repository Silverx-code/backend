mod chess;
mod api;
mod auth;
mod db;

use api::handlers::*;
use auth::handlers::{login_handler, signup_handler};
use auth::models::{LoginRequest, SignupRequest};
use db::create_pool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::Filter;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Create database connection pool
    let db_pool = match create_pool().await {
        Ok(pool) => {
            println!("✅ Database connection established");
            pool
        }
        Err(e) => {
            eprintln!("❌ Failed to create database pool: {}", e);
            std::process::exit(1);
        }
    };

    // Create shared game storage
    let games: GameStore = Arc::new(Mutex::new(HashMap::new()));

    // Create filters
    let games_filter = warp::any().map(move || games.clone());
    let db_filter = warp::any().map(move || db_pool.clone());

    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // ========== AUTH ROUTES ==========

    // POST /api/v1/auth/signup - Register new user
    let signup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("signup"))
        .and(warp::post())
        .and(warp::path::end())
        .and(warp::body::json::<SignupRequest>())
        .and(db_filter.clone())
        .and_then(signup_handler);

    // POST /api/v1/auth/login - User login
    let login = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("login"))
        .and(warp::post())
        .and(warp::path::end())
        .and(warp::body::json::<LoginRequest>())
        .and(db_filter.clone())
        .and_then(login_handler);

    // ========== CHESS GAME ROUTES ==========

    let api = warp::path("api").and(warp::path("v1"));

    // POST /api/v1/games - Create new game
    let new_game = api
        .and(warp::path("games"))
        .and(warp::post())
        .and(warp::path::end())
        .and(games_filter.clone())
        .and_then(create_new_game);

    // GET /api/v1/games/:id - Get game state
    let get_game = api
        .and(warp::path("games"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(warp::path::end())
        .and(games_filter.clone())
        .and_then(get_game_state);

    // POST /api/v1/games/:id/moves - Make a move
    let make_move_route = api
        .and(warp::path("games"))
        .and(warp::path::param::<String>())
        .and(warp::path("moves"))
        .and(warp::post())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(games_filter.clone())
        .and_then(make_move);

    // GET /api/v1/games/:id/moves - Get legal moves
    let get_moves = api
        .and(warp::path("games"))
        .and(warp::path::param::<String>())
        .and(warp::path("moves"))
        .and(warp::get())
        .and(warp::path::end())
        .and(games_filter.clone())
        .and_then(get_legal_moves);

    // GET /api/v1/games/:id/fen - Get game in FEN notation
    let get_fen = api
        .and(warp::path("games"))
        .and(warp::path::param::<String>())
        .and(warp::path("fen"))
        .and(warp::get())
        .and(warp::path::end())
        .and(games_filter.clone())
        .and_then(get_game_fen);

    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "status": "healthy",
                "service": "chess-engine",
                "version": env!("CARGO_PKG_VERSION")
            }))
        });

    // Combine all routes
    let routes = signup
        .or(login)
        .or(new_game)
        .or(get_game)
        .or(make_move_route)
        .or(get_moves)
        .or(get_fen)
        .or(health)
        .with(cors)
        .with(warp::log("chess_engine"));

    println!("🚀 Chess Engine Server starting on http://localhost:3030");
    println!("📋 API Documentation:");
    println!("\n🔐 Authentication:");
    println!("  POST   /api/v1/auth/signup     - Register new user");
    println!("  POST   /api/v1/auth/login      - User login");
    println!("\n♟️  Chess Game:");
    println!("  POST   /api/v1/games           - Create new game");
    println!("  GET    /api/v1/games/:id       - Get game state");
    println!("  POST   /api/v1/games/:id/moves - Make a move");
    println!("  GET    /api/v1/games/:id/moves - Get legal moves");
    println!("  GET    /api/v1/games/:id/fen   - Get FEN notation");
    println!("\n🏥 Health:");
    println!("  GET    /health                 - Health check");

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}