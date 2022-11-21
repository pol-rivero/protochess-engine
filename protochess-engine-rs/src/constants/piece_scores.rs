use crate::types::Centipawns;

pub const KING_SCORE: Centipawns = 320 * CRITICAL_PIECE_MULTIPLIER;
pub const QUEEN_SCORE: Centipawns = 1040;
pub const ROOK_SCORE: Centipawns = 520;
pub const BISHOP_SCORE: Centipawns = 370;
pub const KNIGHT_SCORE: Centipawns = 320;
pub const PAWN_SCORE: Centipawns = 100;

pub const CASTLING_BONUS: Centipawns = 200;
pub const CRITICAL_PIECE_MULTIPLIER: Centipawns = 2;
