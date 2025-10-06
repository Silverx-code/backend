use crate::chess::{GameState, Move};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use warp::Reply;

pub type GameStore = Arc<Mutex<HashMap<String, GameState>>>;

#[derive(Serialize, Deserialize)]
pub struct GameResponse {
    pub game_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct MoveRequest {
    pub from: String, // e.g., "e2"
    pub to: String,   // e.g., "e4"
    pub promotion: Option<String>, // e.g., "Queen"
}

impl MoveRequest {
    pub fn to_move(&self) -> Result<Move, String> {
        let from = crate::chess::Square::from_algebraic(&self.from)
            .ok_or("Invalid source square")?;
        let to = crate::chess::Square::from_algebraic(&self.to)
            .ok_or("Invalid destination square")?;
        
        let mut chess_move = Move::new(from, to);
        
        if let Some(ref promo) = self.promotion {
            let piece_type = match promo.as_str() {
                "Queen" => crate::chess::PieceType::Queen,
                "Rook" => crate::chess::PieceType::Rook,
                "Bishop" => crate::chess::PieceType::Bishop,
                "Knight" => crate::chess::PieceType::Knight,
                _ => return Err("Invalid promotion piece".to_string()),
            };
            chess_move.promotion = Some(piece_type);
        }
        
        // Auto-detect castling
        if (from.rank == 0 || from.rank == 7) && from.file == 4 && (to.file == 6 || to.file == 2) {
            chess_move.is_castling = true;
        }
        
        Ok(chess_move)
    }
}

pub async fn create_new_game(games: GameStore) -> Result<impl Reply, warp::Rejection> {
    let game_id = Uuid::new_v4().to_string();
    let game_state = GameState::new();
    
    {
        let mut games_map = games.lock().unwrap();
        games_map.insert(game_id.clone(), game_state);
    }
    
    let response = GameResponse { game_id };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        warp::http::StatusCode::CREATED,
    ))
}

pub async fn get_game_state(
    game_id: String,
    games: GameStore,
) -> Result<impl Reply, warp::Rejection> {
    let games_map = games.lock().unwrap();
    
    if let Some(game_state) = games_map.get(&game_id) {
        Ok(warp::reply::with_status(
            warp::reply::json(game_state),
            warp::http::StatusCode::OK,
        ))
    } else {
        let error = ErrorResponse {
            error: "Game not found".to_string(),
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&error),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

pub async fn make_move(
    game_id: String,
    move_request: MoveRequest,
    games: GameStore,
) -> Result<impl Reply, warp::Rejection> {
    let chess_move = match move_request.to_move() {
        Ok(m) => m,
        Err(e) => {
            let error = ErrorResponse { error: e };
            return Ok(warp::reply::with_status(
                warp::reply::json(&error),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    let mut games_map = games.lock().unwrap();
    
    if let Some(game_state) = games_map.get_mut(&game_id) {
        match game_state.make_move(chess_move) {
            Ok(()) => {
                Ok(warp::reply::with_status(
                    warp::reply::json(game_state),
                    warp::http::StatusCode::OK,
                ))
            }
            Err(e) => {
                let error = ErrorResponse {
                    error: e.to_string(),
                };
                Ok(warp::reply::with_status(
                    warp::reply::json(&error),
                    warp::http::StatusCode::BAD_REQUEST,
                ))
            }
        }
    } else {
        let error = ErrorResponse {
            error: "Game not found".to_string(),
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&error),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

pub async fn get_legal_moves(
    game_id: String,
    games: GameStore,
) -> Result<impl Reply, warp::Rejection> {
    let games_map = games.lock().unwrap();
    
    if let Some(game_state) = games_map.get(&game_id) {
        let legal_moves = game_state.get_legal_moves();
        
        // Convert moves to a more readable format
        let move_strings: Vec<String> = legal_moves
            .iter()
            .map(|m| format!("{}-{}", m.from.to_algebraic(), m.to.to_algebraic()))
            .collect();
        
        #[derive(Serialize)]
        struct MovesResponse {
            moves: Vec<String>,
            count: usize,
        }
        
        let response = MovesResponse {
            count: move_strings.len(),
            moves: move_strings,
        };
        
        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    } else {
        let error = ErrorResponse {
            error: "Game not found".to_string(),
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&error),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

pub async fn get_game_fen(
    game_id: String,
    games: GameStore,
) -> Result<impl Reply, warp::Rejection> {
    let games_map = games.lock().unwrap();
    
    if let Some(game_state) = games_map.get(&game_id) {
        #[derive(Serialize)]
        struct FenResponse {
            fen: String,
        }
        
        let response = FenResponse {
            fen: game_state.to_fen(),
        };
        
        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    } else {
        let error = ErrorResponse {
            error: "Game not found".to_string(),
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&error),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}