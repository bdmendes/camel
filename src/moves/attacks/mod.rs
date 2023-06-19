use std::collections::HashMap;

use crate::position::{bitboard::Bitboard, board::Piece, square::Square};

use self::sliders::{
    init_slider_blockers_mask, slider_attacks_from_square, BlockersMask, BISHOP_MOVE_DIRECTIONS,
    ROOK_MOVE_DIRECTIONS,
};

pub mod leapers;
pub mod sliders;
pub mod specials;

static ROOK_BLOCKERS_MASK: BlockersMask = init_slider_blockers_mask(&ROOK_MOVE_DIRECTIONS);
static BISHOP_BLOCKERS_MASK: BlockersMask = init_slider_blockers_mask(&BISHOP_MOVE_DIRECTIONS);

//static ROOK_MAGICS: [SquareMagic; 64] = todo!();
//static BISHOP_MAGICS: [SquareMagic; 64] = todo!();

#[derive(Debug, Default)]
struct SquareMagic {
    pub blockers_mask: Bitboard,
    pub shift: u8,
    pub magic_number: Bitboard,
    pub attacks: Vec<Bitboard>,
}

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

fn magic_index(magic: &SquareMagic, occupancy: Bitboard) -> usize {
    let blockers = occupancy & magic.blockers_mask;
    let hash = blockers.wrapping_mul(magic.magic_number.raw());
    (hash >> (64 - magic.shift)) as usize
}

fn sparse_random() -> Bitboard {
    let r1 = rand::random::<u64>();
    let r2 = rand::random::<u64>();
    let r3 = rand::random::<u64>();
    Bitboard::new(r1 & r2 & r3)
}

fn find_magic(square: Square, piece: Piece) -> SquareMagic {
    let blockers_mask = match piece {
        Piece::Rook => ROOK_BLOCKERS_MASK[square as usize],
        Piece::Bishop => BISHOP_BLOCKERS_MASK[square as usize],
        _ => panic!("Only rooks and bishops can have magic bitboards"),
    };
    let shift = blockers_mask.count_ones() as u8;
    let bitsets = bitsets(blockers_mask);
    let mut magic =
        SquareMagic { blockers_mask, shift, magic_number: Bitboard::new(0), attacks: vec![] };

    // Cache moves per bitset
    let mut cached_moves: HashMap<Bitboard, Bitboard> = HashMap::new();
    for bitset in bitsets.iter() {
        let moves = slider_attacks_from_square::<false>(
            square as i8,
            match piece {
                Piece::Rook => &ROOK_MOVE_DIRECTIONS,
                Piece::Bishop => &BISHOP_MOVE_DIRECTIONS,
                _ => unreachable!(),
            },
            Some(*bitset),
        );
        cached_moves.insert(*bitset, moves);
    }

    // Find magic number
    let mut attack_table = vec![Bitboard::new(0); 1 << shift];
    let mut used = vec![false; 1 << shift];
    loop {
        let magic_number = sparse_random();
        magic.magic_number = magic_number;

        let mut found_collision = false;

        for bitset in bitsets.iter() {
            let index = magic_index(&magic, *bitset);
            let moves = *cached_moves.get(bitset).unwrap();

            if used[index] && attack_table[index] != moves {
                found_collision = true;
                break;
            }

            used[index] = true;
            attack_table[index] = moves;
        }

        if !found_collision {
            magic.attacks = attack_table;
            return magic;
        } else {
            used.fill(false);
            continue;
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

        for square in 0..64 {
            let magic = find_magic(Square::try_from(square).unwrap(), Piece::Rook);
            assert_eq!(magic.attacks.len(), 1 << magic.shift);
            println!("{:x} for square {}", magic.magic_number.raw(), square);
        }
    }
}
