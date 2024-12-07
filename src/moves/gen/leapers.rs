use crate::position::{
    bitboard::Bitboard,
    square::{Direction, Square},
};

pub type LeaperAttackMap = [Bitboard; 64];

pub const fn init_leaper_attacks(move_directions: &[Direction]) -> LeaperAttackMap {
    let mut attacks: LeaperAttackMap = [Bitboard::new(0); 64];

    let mut square = 0;
    while square < 64 {
        let file = square % 8;
        let mut bb = 0;

        let mut i = 0;
        while i < move_directions.len() {
            let target_square = square + move_directions[i];
            let target_square_file = target_square % 8;
            if target_square >= 0 && target_square < 64 && (target_square_file - file).abs() <= 2 {
                bb |= 1 << target_square;
            }
            i += 1;
        }

        attacks[square as usize] = Bitboard::new(bb);
        square += 1
    }

    attacks
}

static KNIGHT_ATTACKS: LeaperAttackMap = init_leaper_attacks(&[
    Square::NORTH + 2 * Square::WEST,
    Square::NORTH + 2 * Square::EAST,
    Square::SOUTH + 2 * Square::WEST,
    Square::SOUTH + 2 * Square::EAST,
    2 * Square::NORTH + Square::WEST,
    2 * Square::NORTH + Square::EAST,
    2 * Square::SOUTH + Square::WEST,
    2 * Square::SOUTH + Square::EAST,
]);

static KING_ATTACKS: LeaperAttackMap = init_leaper_attacks(&[
    Square::NORTH,
    Square::NORTH + Square::EAST,
    Square::EAST,
    Square::SOUTH + Square::EAST,
    Square::SOUTH,
    Square::SOUTH + Square::WEST,
    Square::WEST,
    Square::NORTH + Square::WEST,
]);
