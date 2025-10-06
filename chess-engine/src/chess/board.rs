use super::types::{Color, Piece, PieceType, Square};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    squares: [[Option<Piece>; 8]; 8],
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            squares: [[None; 8]; 8],
        };
        board.setup_starting_position();
        board
    }

    pub fn empty() -> Self {
        Self {
            squares: [[None; 8]; 8],
        }
    }

    fn setup_starting_position(&mut self) {
        // White pieces
        self.set_piece(Square::new(0, 0).unwrap(), Piece::new(PieceType::Rook, Color::White));
        self.set_piece(Square::new(1, 0).unwrap(), Piece::new(PieceType::Knight, Color::White));
        self.set_piece(Square::new(2, 0).unwrap(), Piece::new(PieceType::Bishop, Color::White));
        self.set_piece(Square::new(3, 0).unwrap(), Piece::new(PieceType::King, Color::White));
        self.set_piece(Square::new(4, 0).unwrap(), Piece::new(PieceType::Queen, Color::White));
        self.set_piece(Square::new(5, 0).unwrap(), Piece::new(PieceType::Bishop, Color::White));
        self.set_piece(Square::new(6, 0).unwrap(), Piece::new(PieceType::Knight, Color::White));
        self.set_piece(Square::new(7, 0).unwrap(), Piece::new(PieceType::Rook, Color::White));

        // White pawns
        for file in 0..8 {
            self.set_piece(Square::new(file, 1).unwrap(), Piece::new(PieceType::Pawn, Color::White));
        }

        // Black pieces
        self.set_piece(Square::new(0, 7).unwrap(), Piece::new(PieceType::Rook, Color::Black));
        self.set_piece(Square::new(1, 7).unwrap(), Piece::new(PieceType::Knight, Color::Black));
        self.set_piece(Square::new(2, 7).unwrap(), Piece::new(PieceType::Bishop, Color::Black));
        self.set_piece(Square::new(3, 7).unwrap(), Piece::new(PieceType::King, Color::Black));
        self.set_piece(Square::new(4, 7).unwrap(), Piece::new(PieceType::Queen, Color::Black));
        self.set_piece(Square::new(5, 7).unwrap(), Piece::new(PieceType::Bishop, Color::Black));
        self.set_piece(Square::new(6, 7).unwrap(), Piece::new(PieceType::Knight, Color::Black));
        self.set_piece(Square::new(7, 7).unwrap(), Piece::new(PieceType::Rook, Color::Black));

        // Black pawns
        for file in 0..8 {
            self.set_piece(Square::new(file, 6).unwrap(), Piece::new(PieceType::Pawn, Color::Black));
        }
    }

    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        if square.is_valid() {
            self.squares[square.rank as usize][square.file as usize]
        } else {
            None
        }
    }

    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        if square.is_valid() {
            self.squares[square.rank as usize][square.file as usize] = Some(piece);
        }
    }

    pub fn remove_piece(&mut self, square: Square) -> Option<Piece> {
        if square.is_valid() {
            let piece = self.squares[square.rank as usize][square.file as usize];
            self.squares[square.rank as usize][square.file as usize] = None;
            piece
        } else {
            None
        }
    }

    pub fn move_piece(&mut self, from: Square, to: Square) -> Option<Piece> {
        let piece = self.remove_piece(from)?;
        let captured = self.remove_piece(to);
        self.set_piece(to, piece);
        captured
    }

    pub fn find_king(&self, color: Color) -> Option<Square> {
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank).unwrap();
                if let Some(piece) = self.get_piece(square) {
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return Some(square);
                    }
                }
            }
        }
        None
    }

    pub fn get_pieces(&self, color: Color) -> Vec<(Square, Piece)> {
        let mut pieces = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank).unwrap();
                if let Some(piece) = self.get_piece(square) {
                    if piece.color == color {
                        pieces.push((square, piece));
                    }
                }
            }
        }
        pieces
    }

    pub fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        // Check if any piece of the given color can attack the square
        for rank in 0..8 {
            for file in 0..8 {
                let from = Square::new(file, rank).unwrap();
                if let Some(piece) = self.get_piece(from) {
                    if piece.color == by_color && self.can_piece_attack(from, square, piece) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn can_piece_attack(&self, from: Square, to: Square, piece: Piece) -> bool {
        if from == to {
            return false;
        }

        let file_diff = (to.file as i8 - from.file as i8).abs();
        let rank_diff = (to.rank as i8 - from.rank as i8).abs();

        match piece.piece_type {
            PieceType::Pawn => self.can_pawn_attack(from, to, piece.color),
            PieceType::Rook => {
                (file_diff == 0 || rank_diff == 0) && self.is_path_clear(from, to)
            }
            PieceType::Bishop => {
                file_diff == rank_diff && self.is_path_clear(from, to)
            }
            PieceType::Queen => {
                (file_diff == 0 || rank_diff == 0 || file_diff == rank_diff) 
                && self.is_path_clear(from, to)
            }
            PieceType::Knight => {
                (file_diff == 2 && rank_diff == 1) || (file_diff == 1 && rank_diff == 2)
            }
            PieceType::King => {
                file_diff <= 1 && rank_diff <= 1
            }
        }
    }

    fn can_pawn_attack(&self, from: Square, to: Square, color: Color) -> bool {
        let direction = match color {
            Color::White => 1,
            Color::Black => -1,
        };

        let file_diff = to.file as i8 - from.file as i8;
        let rank_diff = to.rank as i8 - from.rank as i8;

        // Pawn attacks diagonally one square
        file_diff.abs() == 1 && rank_diff == direction
    }

    pub fn is_path_clear(&self, from: Square, to: Square) -> bool {
        let file_step = (to.file as i8 - from.file as i8).signum();
        let rank_step = (to.rank as i8 - from.rank as i8).signum();

        let mut current_file = from.file as i8 + file_step;
        let mut current_rank = from.rank as i8 + rank_step;

        while current_file != to.file as i8 || current_rank != to.rank as i8 {
            let square = Square::new(current_file as u8, current_rank as u8).unwrap();
            if self.get_piece(square).is_some() {
                return false;
            }
            current_file += file_step;
            current_rank += rank_step;
        }

        true
    }

    pub fn to_2d_array(&self) -> [[Option<Piece>; 8]; 8] {
        self.squares
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}