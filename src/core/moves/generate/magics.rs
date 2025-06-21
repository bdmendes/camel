use super::sliders::{BISHOP_MOVE_DIRECTIONS, ROOK_MOVE_DIRECTIONS, slider_attacks_from_square};
use crate::core::position::{Position, bitboard::Bitboard, piece::Piece, square::Square};
use ctor::ctor;
use rand::{RngCore, SeedableRng, rngs::StdRng};
use std::thread;

#[ctor]
static BISHOP_MAGICS: [SquareMagic; 64] = { find_magics(Piece::Bishop) };

#[ctor]
static ROOK_MAGICS: [SquareMagic; 64] = { find_magics(Piece::Rook) };

#[derive(Debug)]
struct SquareMagic {
    shift: u8,
    mask: u64,
    magic: u64,
    attacks: Vec<Bitboard>,
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

fn sparse_random(seed: u64) -> u64 {
    let mut rng = StdRng::seed_from_u64(seed);
    let r1 = rng.next_u64();
    let r2 = rng.next_u64();
    let r3 = rng.next_u64();
    r1 & r2 & r3
}

fn magic_index(occupancy: Bitboard, magic: &SquareMagic) -> usize {
    let occupancy = occupancy.raw() & magic.mask;
    let index = (occupancy.wrapping_mul(magic.magic)) >> (64 - magic.shift);
    index as usize
}

fn find_magic(square: Square, piece: Piece) -> SquareMagic {
    let directions = match piece {
        Piece::Bishop => &BISHOP_MOVE_DIRECTIONS,
        Piece::Rook => &ROOK_MOVE_DIRECTIONS,
        _ => panic!("There are no magics for that piece."),
    };
    let blockers_mask = slider_attacks_from_square(square, directions, Bitboard::empty(), true);
    let shift = blockers_mask.count_ones() as u8;

    let occ_move_map = bitsets(blockers_mask)
        .iter()
        .map(|b| {
            (
                *b,
                slider_attacks_from_square(square, directions, *b, false),
            )
        })
        .collect::<Vec<_>>();

    let mut magic_tentative = SquareMagic {
        shift,
        mask: blockers_mask.raw(),
        magic: 0,
        attacks: vec![Bitboard::empty(); 1 << shift],
    };

    for seed in 0.. {
        magic_tentative.magic = sparse_random(seed);
        let mut used = [false; 4096];
        let mut found_collision = false;

        for (occupancy, moves) in &occ_move_map {
            let index = magic_index(*occupancy, &magic_tentative);
            if used[index] && magic_tentative.attacks[index] != *moves {
                found_collision = true;
                break;
            }
            magic_tentative.attacks[index] = *moves;
            used[index] = true;
        }

        if !found_collision {
            let largest_used_index = used.iter().rposition(|&used| used).unwrap();
            magic_tentative
                .attacks
                .resize(largest_used_index + 1, Bitboard::empty());
            return magic_tentative;
        }
    }

    panic!("Could not find any valid magic using u64 seeds.");
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

pub fn bishop_attacks(position: &Position, square: Square) -> Bitboard {
    let magic = &BISHOP_MAGICS[square as usize];
    let index = magic_index(position.occupancy_bb_all(), magic);
    magic.attacks[index]
}

pub fn rook_attacks(position: &Position, square: Square) -> Bitboard {
    let magic = &ROOK_MAGICS[square as usize];
    let index = magic_index(position.occupancy_bb_all(), magic);
    magic.attacks[index]
}

pub fn queen_attacks(position: &Position, square: Square) -> Bitboard {
    bishop_attacks(position, square) | rook_attacks(position, square)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::bitsets;
    use crate::{
        core::moves::generate::magics::{bishop_attacks, rook_attacks},
        core::position::{Position, bitboard::Bitboard, square::Square},
    };

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
    fn bishop_attack() {
        let position =
            Position::from_str("r2k3r/p3ppb1/6p1/2RPPn1p/Qn3Pb1/2N2N2/1P4PP/2B1K2R w K - 2 17")
                .unwrap();

        assert_eq!(
            bishop_attacks(&position, Square::G7),
            Bitboard::from_square(Square::F6)
                | Bitboard::from_square(Square::E5)
                | Bitboard::from_square(Square::H8)
                | Bitboard::from_square(Square::H6)
                | Bitboard::from_square(Square::F8)
        );
    }

    #[test]
    fn rook_attack() {
        let position =
            Position::from_str("r2kQ2r/p3ppb1/6p1/2RPPn1p/1n3Pb1/2N2N2/1P4PP/2B1K2R b K - 3 17")
                .unwrap();

        assert_eq!(
            rook_attacks(&position, Square::H8),
            Bitboard::from_square(Square::H7)
                | Bitboard::from_square(Square::H6)
                | Bitboard::from_square(Square::H5)
                | Bitboard::from_square(Square::G8)
                | Bitboard::from_square(Square::F8)
                | Bitboard::from_square(Square::E8)
        );
    }
}
