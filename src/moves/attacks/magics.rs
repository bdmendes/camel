use super::sliders::{slider_attacks_from_square, BISHOP_MOVE_DIRECTIONS, ROOK_MOVE_DIRECTIONS};
use crate::position::{bitboard::Bitboard, board::Piece, square::Square};
use ctor::ctor;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::thread;

#[ctor]
pub static ROOK_MAGICS: [SquareMagic; 64] = find_magics(Piece::Rook);

#[ctor]
pub static BISHOP_MAGICS: [SquareMagic; 64] = find_magics(Piece::Bishop);

#[derive(Debug, Default)]
pub struct SquareMagic {
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

fn sparse_random(seed: u64) -> Bitboard {
    let mut rng = StdRng::seed_from_u64(seed);
    let r1 = rng.gen::<u64>();
    let r2 = rng.gen::<u64>();
    let r3 = rng.gen::<u64>();

    Bitboard::new(r1 & r2 & r3)
}

fn find_magic(square: Square, piece: Piece) -> SquareMagic {
    let directions = match piece {
        Piece::Rook => &ROOK_MOVE_DIRECTIONS,
        Piece::Bishop => &BISHOP_MOVE_DIRECTIONS,
        _ => panic!("Invalid piece"),
    };

    let blockers_mask = slider_attacks_from_square::<true>(square, directions, None);
    let shift = blockers_mask.count_ones() as u8;

    let mut magic = SquareMagic {
        blockers_mask,
        shift,
        magic_number: Bitboard::new(0),
        attacks: vec![Bitboard::new(0); 1 << shift],
    };

    let bitsets = bitsets(blockers_mask);
    let moves = bitsets
        .iter()
        .map(|bitset| slider_attacks_from_square::<false>(square, directions, Some(*bitset)))
        .collect::<Vec<_>>();

    for seed in 0.. {
        magic.magic_number = sparse_random(seed);

        let mut found_collision = false;
        let mut used = vec![false; 1 << shift];

        for (i, bitset) in bitsets.iter().enumerate() {
            let index = magic_index(&magic, *bitset);

            if used[index] && magic.attacks[index] != moves[i] {
                found_collision = true;
                break;
            }

            used[index] = true;
            magic.attacks[index] = moves[i];
        }

        if !found_collision {
            let largest_used_index =
                used.iter().enumerate().filter(|(_, used)| **used).map(|(i, _)| i).max().unwrap();
            magic.attacks.resize(largest_used_index + 1, Bitboard::new(0));
            return magic;
        }
    }

    panic!("Magic not found");
}

fn find_magics(piece: Piece) -> [SquareMagic; 64] {
    (0..64)
        .map(|square| thread::spawn(move || find_magic(Square::from(square).unwrap(), piece)))
        .collect::<Vec<_>>()
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

pub fn magic_index(magic: &SquareMagic, occupancy: Bitboard) -> usize {
    let blockers = occupancy & magic.blockers_mask;
    let hash = blockers.wrapping_mul(magic.magic_number.raw());
    (hash >> (64 - magic.shift)) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_magics(piece: Piece) {
        let directions = match piece {
            Piece::Rook => &ROOK_MOVE_DIRECTIONS,
            Piece::Bishop => &BISHOP_MOVE_DIRECTIONS,
            _ => panic!("Invalid piece"),
        };

        for square in Square::list() {
            let magic = find_magic(*square, piece);

            let blockers_mask = slider_attacks_from_square::<true>(*square, directions, None);
            let bitsets = bitsets(blockers_mask);

            for bitset in bitsets {
                let index = magic_index(&magic, bitset);
                assert_eq!(
                    magic.attacks[index],
                    slider_attacks_from_square::<false>(*square, directions, Some(bitset),)
                );
            }
        }
    }

    #[test]
    fn bitsets_simple() {
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
    fn rook_magics() {
        test_magics(Piece::Rook);
    }

    #[test]
    fn bishop_magics() {
        test_magics(Piece::Bishop);
    }
}
