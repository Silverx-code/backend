use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Self { piece_type, color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Square {
    pub file: u8, // 0-7 representing a-h
    pub rank: u8, // 0-7 representing 1-8
}

impl Square {
    pub fn new(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Self { file, rank })
        } else {
            None
        }
    }

    pub fn from_algebraic(notation: &str) -> Option<Self> {
        if notation.len() != 2 {
            return None;
        }

        let chars: Vec<char> = notation.chars().collect();
        let file = (chars[0] as u8).checked_sub(b'a')?;
        let rank = (chars[1] as u8).checked_sub(b'1')?;

        Self::new(file, rank)
    }

    pub fn to_algebraic(self) -> String {
        format!("{}{}", (b'a' + self.file) as char, (b'1' + self.rank) as char)
    }

    pub fn is_valid(self) -> bool {
        self.file < 8 && self.rank < 8
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceType>,
    pub is_castling: bool,
    pub is_en_passant: bool,
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_castling: false,
            is_en_passant: false,
        }
    }

    pub fn with_promotion(mut self, piece_type: PieceType) -> Self {
        self.promotion = Some(piece_type);
        self
    }

    pub fn castling(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_castling: true,
            is_en_passant: false,
        }
    }

    pub fn en_passant(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_castling: false,
            is_en_passant: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn new() -> Self {
        Self {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }

    pub fn can_castle(&self, color: Color, kingside: bool) -> bool {
        match (color, kingside) {
            (Color::White, true) => self.white_kingside,
            (Color::White, false) => self.white_queenside,
            (Color::Black, true) => self.black_kingside,
            (Color::Black, false) => self.black_queenside,
        }
    }

    pub fn remove_rights(&mut self, color: Color, kingside: Option<bool>) {
        match color {
            Color::White => {
                match kingside {
                    Some(true) => self.white_kingside = false,
                    Some(false) => self.white_queenside = false,
                    None => {
                        self.white_kingside = false;
                        self.white_queenside = false;
                    }
                }
            }
            Color::Black => {
                match kingside {
                    Some(true) => self.black_kingside = false,
                    Some(false) => self.black_queenside = false,
                    None => {
                        self.black_kingside = false;
                        self.black_queenside = false;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    InProgress,
    Check,
    Checkmate(Color), // Winner
    Stalemate,
    Draw,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self::new()
    }
}