use rand::random;

use crate::position::{bitboard::Bitboard, board::Piece, square::Square};

use super::{
    attacks::rook::{init_rook_blockers_mask, rook_attacks_from_square},
    gen::AttackMap,
};

#[derive(Debug, Copy, Clone, Default)]
struct Magic {
    pub mask: Bitboard,
    pub magic: Bitboard,
    pub shift: u8,
}

impl Magic {
    pub const fn new(mask: Bitboard, magic: Bitboard, shift: u8) -> Self {
        Magic { mask, magic, shift }
    }
}

const ROOK_BLOCKER_MASK: AttackMap = init_rook_blockers_mask();

const ROOK_MAGICS: [Magic; 64] = [Magic::new(Bitboard::new(0), Bitboard::new(0), 0); 64];
const BISHOP_MAGICS: [Magic; 64] = [Magic::new(Bitboard::new(0), Bitboard::new(0), 0); 64];

const ROOK_ATTACKS: [Bitboard; 4096 * 64] = [Bitboard::new(0); 4096 * 64];
const BISHOP_ATTACKS: [Bitboard; 512 * 64] = [Bitboard::new(0); 512 * 64];

fn bitsets(bitboard: Bitboard) -> Vec<Bitboard> {
    let bitboard = bitboard.raw();
    let mut bitsets = Vec::new();
    let mut current_bb = 0;

    loop {
        bitsets.push(Bitboard::new(current_bb));
        current_bb = (current_bb.wrapping_sub(bitboard)) & bitboard;
        if current_bb == 0 {
            break;
        }
    }

    bitsets
}

fn magic_index(magic: &Magic, occupancy: Bitboard) -> usize {
    let blockers = occupancy & magic.mask;
    let hash = blockers.wrapping_mul(magic.magic.raw());
    (hash >> magic.shift) as usize
}

fn find_magic<const MAX_INDEX: usize>(
    piece: Piece,
    square: Square,
) -> (Magic, [Bitboard; MAX_INDEX]) {
    let blocker_mask = match piece {
        Piece::Rook => ROOK_BLOCKER_MASK[square as usize],
        Piece::Bishop => todo!(),
        _ => panic!("Invalid piece"),
    };
    let shift = 64 - blocker_mask.count_ones() as u8;
    let bitsets = bitsets(blocker_mask);

    loop {
        let magic_number = random::<u64>() & random::<u64>() & random::<u64>();
        let magic = Magic::new(blocker_mask, Bitboard::new(magic_number), shift);

        let mut moves = [Bitboard::new(std::u64::MAX); MAX_INDEX];
        let mut found_collision = false;

        for bitset in &bitsets {
            let index = magic_index(&magic, *bitset);
            if index >= MAX_INDEX {
                found_collision = true;
            }

            if found_collision {
                break;
            }

            let piece_moves = match piece {
                Piece::Rook => rook_attacks_from_square::<false>(square as i32, Some(*bitset)),
                Piece::Bishop => todo!(),
                _ => panic!("Invalid piece"),
            };

            if moves[index] != Bitboard::new(std::u64::MAX) && moves[index] != piece_moves {
                found_collision = true;
                break;
            }

            moves[index] = piece_moves;
        }

        if !found_collision {
            return (magic, moves);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bitsets() {
        use super::*;

        let bitboard = Bitboard::new(0b11001);
        let bitsets = bitsets(bitboard);

        let expected_bitsets = [
            Bitboard::new(0b0),
            Bitboard::new(0b1),
            Bitboard::new(0b1000),
            Bitboard::new(0b1001),
            Bitboard::new(0b10000),
            Bitboard::new(0b10001),
            Bitboard::new(0b11000),
            Bitboard::new(0b11001),
        ];

        for bitset in bitsets {
            assert!(expected_bitsets.contains(&bitset));
        }
    }

    #[test]
    fn finds_magic() {
        use super::*;

        let magic = find_magic::<10000>(Piece::Rook, Square::E4);
        println!("{:?}", magic);
    }
}
