pub mod types;
pub mod board;
pub mod game;

// Re-export all types for easier access
pub use types::{Color, Piece, PieceType, Square, Move, CastlingRights, GameStatus};
pub use board::Board;
pub use game::{GameState, ChessError};