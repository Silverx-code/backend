use super::{board::Board, types::*};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChessError {
    #[error("Invalid move: {0}")]
    InvalidMove(String),
    #[error("Game is over")]
    GameOver,
    #[error("Not your turn")]
    NotYourTurn,
    #[error("King would be in check")]
    KingInCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: Board,
    pub current_player: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_target: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub status: GameStatus,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_player: Color::White,
            castling_rights: CastlingRights::new(),
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            status: GameStatus::InProgress,
        }
    }

    pub fn make_move(&mut self, chess_move: Move) -> Result<(), ChessError> {
        // Check if game is over
        match self.status {
            GameStatus::Checkmate(_) | GameStatus::Stalemate | GameStatus::Draw => {
                return Err(ChessError::GameOver);
            }
            _ => {}
        }

        // Validate the move
        self.validate_move(&chess_move)?;

        // Make the move
        self.execute_move(chess_move.clone());

        // Update game state
        self.update_castling_rights(&chess_move);
        self.update_en_passant(&chess_move);
        self.update_clocks(&chess_move);
        self.switch_player();
        self.update_status();

        Ok(())
    }

    fn validate_move(&self, chess_move: &Move) -> Result<(), ChessError> {
        // Check if piece exists at source
        let piece = self.board.get_piece(chess_move.from)
            .ok_or_else(|| ChessError::InvalidMove("No piece at source square".to_string()))?;

        // Check if it's the current player's piece
        if piece.color != self.current_player {
            return Err(ChessError::NotYourTurn);
        }

        // Check if the move is legal for this piece type
        if !self.is_legal_move(chess_move, piece) {
            return Err(ChessError::InvalidMove("Illegal move for this piece".to_string()));
        }

        // Check if move would leave king in check
        if self.would_leave_king_in_check(chess_move) {
            return Err(ChessError::KingInCheck);
        }

        Ok(())
    }

    fn is_legal_move(&self, chess_move: &Move, piece: Piece) -> bool {
        let from = chess_move.from;
        let to = chess_move.to;

        // Check if destination has same color piece
        if let Some(dest_piece) = self.board.get_piece(to) {
            if dest_piece.color == piece.color {
                return false;
            }
        }

        match piece.piece_type {
            PieceType::Pawn => self.is_legal_pawn_move(chess_move, piece.color),
            PieceType::Rook => self.is_legal_rook_move(from, to),
            PieceType::Knight => self.is_legal_knight_move(from, to),
            PieceType::Bishop => self.is_legal_bishop_move(from, to),
            PieceType::Queen => self.is_legal_queen_move(from, to),
            PieceType::King => {
                if chess_move.is_castling {
                    self.is_legal_castling(chess_move, piece.color)
                } else {
                    self.is_legal_king_move(from, to)
                }
            }
        }
    }

    fn is_legal_pawn_move(&self, chess_move: &Move, color: Color) -> bool {
        let from = chess_move.from;
        let to = chess_move.to;
        
        let direction = match color {
            Color::White => 1,
            Color::Black => -1,
        };

        let file_diff = to.file as i8 - from.file as i8;
        let rank_diff = to.rank as i8 - from.rank as i8;

        // Forward moves
        if file_diff == 0 {
            if rank_diff == direction {
                // One square forward
                self.board.get_piece(to).is_none()
            } else if rank_diff == 2 * direction {
                // Two squares forward from starting position
                let starting_rank = match color {
                    Color::White => 1,
                    Color::Black => 6,
                };
                from.rank == starting_rank 
                    && self.board.get_piece(to).is_none()
                    && self.board.is_path_clear(from, to)
            } else {
                false
            }
        } else if file_diff.abs() == 1 && rank_diff == direction {
            // Diagonal capture
            if self.board.get_piece(to).is_some() {
                true // Regular capture
            } else if Some(to) == self.en_passant_target {
                chess_move.is_en_passant // En passant
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_legal_rook_move(&self, from: Square, to: Square) -> bool {
        let file_diff = (to.file as i8 - from.file as i8).abs();
        let rank_diff = (to.rank as i8 - from.rank as i8).abs();

        (file_diff == 0 || rank_diff == 0) && self.board.is_path_clear(from, to)
    }

    fn is_legal_knight_move(&self, from: Square, to: Square) -> bool {
        let file_diff = (to.file as i8 - from.file as i8).abs();
        let rank_diff = (to.rank as i8 - from.rank as i8).abs();

        (file_diff == 2 && rank_diff == 1) || (file_diff == 1 && rank_diff == 2)
    }

    fn is_legal_bishop_move(&self, from: Square, to: Square) -> bool {
        let file_diff = (to.file as i8 - from.file as i8).abs();
        let rank_diff = (to.rank as i8 - from.rank as i8).abs();

        file_diff == rank_diff && file_diff > 0 && self.board.is_path_clear(from, to)
    }

    fn is_legal_queen_move(&self, from: Square, to: Square) -> bool {
        self.is_legal_rook_move(from, to) || self.is_legal_bishop_move(from, to)
    }

    fn is_legal_king_move(&self, from: Square, to: Square) -> bool {
        let file_diff = (to.file as i8 - from.file as i8).abs();
        let rank_diff = (to.rank as i8 - from.rank as i8).abs();

        file_diff <= 1 && rank_diff <= 1 && (file_diff > 0 || rank_diff > 0)
    }

    fn is_legal_castling(&self, chess_move: &Move, color: Color) -> bool {
        let from = chess_move.from;
        let to = chess_move.to;

        // Check if king is on starting square
        let king_start_square = match color {
            Color::White => Square::new(4, 0).unwrap(),
            Color::Black => Square::new(4, 7).unwrap(),
        };

        if from != king_start_square {
            return false;
        }

        // Determine if kingside or queenside
        let kingside = to.file > from.file;

        // Check castling rights
        if !self.castling_rights.can_castle(color, kingside) {
            return false;
        }

        // Check if king is in check
        if self.is_in_check(color) {
            return false;
        }

        // Check if path is clear and king doesn't move through check
        let squares_to_check = if kingside {
            vec![
                Square::new(5, from.rank).unwrap(),
                Square::new(6, from.rank).unwrap(),
            ]
        } else {
            vec![
                Square::new(3, from.rank).unwrap(),
                Square::new(2, from.rank).unwrap(),
                Square::new(1, from.rank).unwrap(),
            ]
        };

        for square in squares_to_check {
            if square != to && self.board.get_piece(square).is_some() {
                return false; // Path not clear
            }
            if square.file <= 6 && square.file >= 2 {
                // Check if king would be in check on this square
                let mut temp_board = self.board.clone();
                temp_board.move_piece(from, square);
                if temp_board.is_square_attacked(square, color.opposite()) {
                    return false;
                }
            }
        }

        true
    }

    fn would_leave_king_in_check(&self, chess_move: &Move) -> bool {
        // Make a temporary copy of the board
        let mut temp_board = self.board.clone();
        
        // Execute the move on the temporary board
        let piece = temp_board.get_piece(chess_move.from).unwrap();
        temp_board.move_piece(chess_move.from, chess_move.to);
        
        // Handle en passant capture
        if chess_move.is_en_passant {
            let capture_square = Square::new(
                chess_move.to.file,
                chess_move.from.rank,
            ).unwrap();
            temp_board.remove_piece(capture_square);
        }

        // Find king position
        let king_square = if piece.piece_type == PieceType::King {
            chess_move.to
        } else {
            temp_board.find_king(self.current_player).unwrap()
        };

        // Check if king is attacked
        temp_board.is_square_attacked(king_square, self.current_player.opposite())
    }

    fn execute_move(&mut self, chess_move: Move) {
        let piece = self.board.get_piece(chess_move.from).unwrap();

        if chess_move.is_castling {
            // Move king
            self.board.move_piece(chess_move.from, chess_move.to);
            
            // Move rook
            let (rook_from, rook_to) = if chess_move.to.file > chess_move.from.file {
                // Kingside castling
                (Square::new(7, chess_move.from.rank).unwrap(), 
                 Square::new(5, chess_move.from.rank).unwrap())
            } else {
                // Queenside castling
                (Square::new(0, chess_move.from.rank).unwrap(), 
                 Square::new(3, chess_move.from.rank).unwrap())
            };
            self.board.move_piece(rook_from, rook_to);
        } else {
            // Regular move
            self.board.move_piece(chess_move.from, chess_move.to);
            
            // Handle en passant capture
            if chess_move.is_en_passant {
                let capture_square = Square::new(
                    chess_move.to.file,
                    chess_move.from.rank,
                ).unwrap();
                self.board.remove_piece(capture_square);
            }
            
            // Handle pawn promotion
            if let Some(promotion) = chess_move.promotion {
                self.board.set_piece(chess_move.to, Piece::new(promotion, piece.color));
            }
        }
    }

    fn update_castling_rights(&mut self, chess_move: &Move) {
        let piece = self.board.get_piece(chess_move.to).unwrap();
        
        match piece.piece_type {
            PieceType::King => {
                self.castling_rights.remove_rights(piece.color, None);
            }
            PieceType::Rook => {
                // Check if rook moved from starting position
                let (queenside_file, kingside_file, rank) = match piece.color {
                    Color::White => (0, 7, 0),
                    Color::Black => (0, 7, 7),
                };
                
                if chess_move.from == Square::new(queenside_file, rank).unwrap() {
                    self.castling_rights.remove_rights(piece.color, Some(false));
                } else if chess_move.from == Square::new(kingside_file, rank).unwrap() {
                    self.castling_rights.remove_rights(piece.color, Some(true));
                }
            }
            _ => {}
        }
    }

    fn update_en_passant(&mut self, chess_move: &Move) {
        let piece = self.board.get_piece(chess_move.to).unwrap();
        
        // Reset en passant target
        self.en_passant_target = None;
        
        // Check if pawn moved two squares
        if piece.piece_type == PieceType::Pawn {
            let rank_diff = (chess_move.to.rank as i8 - chess_move.from.rank as i8).abs();
            if rank_diff == 2 {
                // Set en passant target square
                let target_rank = (chess_move.from.rank + chess_move.to.rank) / 2;
                self.en_passant_target = Some(Square::new(chess_move.to.file, target_rank).unwrap());
            }
        }
    }

    fn update_clocks(&mut self, chess_move: &Move) {
        let piece = self.board.get_piece(chess_move.to).unwrap();
        
        // Reset halfmove clock on pawn move or capture
        if piece.piece_type == PieceType::Pawn || chess_move.is_en_passant {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
    }

    fn switch_player(&mut self) {
        self.current_player = self.current_player.opposite();
        if self.current_player == Color::White {
            self.fullmove_number += 1;
        }
    }

    fn update_status(&mut self) {
        let in_check = self.is_in_check(self.current_player);
        let has_legal_moves = self.has_legal_moves();

        self.status = if !has_legal_moves {
            if in_check {
                GameStatus::Checkmate(self.current_player.opposite())
            } else {
                GameStatus::Stalemate
            }
        } else if in_check {
            GameStatus::Check
        } else {
            GameStatus::InProgress
        };

        // Check for draw conditions
        if self.halfmove_clock >= 50 {
            self.status = GameStatus::Draw;
        }
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        if let Some(king_square) = self.board.find_king(color) {
            self.board.is_square_attacked(king_square, color.opposite())
        } else {
            false
        }
    }

    fn has_legal_moves(&self) -> bool {
        let pieces = self.board.get_pieces(self.current_player);
        
        for (from, piece) in pieces {
            for rank in 0..8 {
                for file in 0..8 {
                    let to = Square::new(file, rank).unwrap();
                    let chess_move = Move::new(from, to);
                    
                    if self.is_legal_move(&chess_move, piece) && !self.would_leave_king_in_check(&chess_move) {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let pieces = self.board.get_pieces(self.current_player);
        
        for (from, piece) in pieces {
            for rank in 0..8 {
                for file in 0..8 {
                    let to = Square::new(file, rank).unwrap();
                    let mut chess_move = Move::new(from, to);
                    
                    // Check for castling
                    if piece.piece_type == PieceType::King {
                        let file_diff = to.file as i8 - from.file as i8;
                        if file_diff.abs() == 2 {
                            chess_move.is_castling = true;
                        }
                    }
                    
                    // Check for en passant
                    if piece.piece_type == PieceType::Pawn && Some(to) == self.en_passant_target {
                        chess_move.is_en_passant = true;
                    }
                    
                    if self.is_legal_move(&chess_move, piece) && !self.would_leave_king_in_check(&chess_move) {
                        // Check for pawn promotion
                        if piece.piece_type == PieceType::Pawn {
                            let promotion_rank = match piece.color {
                                Color::White => 7,
                                Color::Black => 0,
                            };
                            
                            if to.rank == promotion_rank {
                                // Add all possible promotions
                                for promotion in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                                    let mut promo_move = chess_move.clone();
                                    promo_move.promotion = Some(promotion);
                                    moves.push(promo_move);
                                }
                            } else {
                                moves.push(chess_move);
                            }
                        } else {
                            moves.push(chess_move);
                        }
                    }
                }
            }
        }
        
        moves
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        
        // Piece placement
        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let square = Square::new(file, rank).unwrap();
                if let Some(piece) = self.board.get_piece(square) {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    let piece_char = match piece.piece_type {
                        PieceType::Pawn => 'p',
                        PieceType::Rook => 'r',
                        PieceType::Knight => 'n',
                        PieceType::Bishop => 'b',
                        PieceType::Queen => 'q',
                        PieceType::King => 'k',
                    };
                    let piece_char = match piece.color {
                        Color::White => piece_char.to_ascii_uppercase(),
                        Color::Black => piece_char,
                    };
                    fen.push(piece_char);
                } else {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }
        
        // Active color
        fen.push(' ');
        fen.push(match self.current_player {
            Color::White => 'w',
            Color::Black => 'b',
        });
        
        // Castling rights
        fen.push(' ');
        let mut castling = String::new();
        if self.castling_rights.white_kingside { castling.push('K'); }
        if self.castling_rights.white_queenside { castling.push('Q'); }
        if self.castling_rights.black_kingside { castling.push('k'); }
        if self.castling_rights.black_queenside { castling.push('q'); }
        if castling.is_empty() { castling.push('-'); }
        fen.push_str(&castling);
        
        // En passant target
        fen.push(' ');
        if let Some(target) = self.en_passant_target {
            fen.push_str(&target.to_algebraic());
        } else {
            fen.push('-');
        }
        
        // Halfmove clock and fullmove number
        fen.push_str(&format!(" {} {}", self.halfmove_clock, self.fullmove_number));
        
        fen
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}