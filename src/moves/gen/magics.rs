use super::sliders::{slider_attacks_from_square, BISHOP_MOVE_DIRECTIONS, ROOK_MOVE_DIRECTIONS};
use crate::position::{bitboard::Bitboard, piece::Piece, square::Square};
use ctor::ctor;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::thread;

#[ctor]
static BISHOP_MAGICS: [SquareMagic; 64] = find_magics(Piece::Bishop);

#[ctor]
static ROOK_MAGICS: [SquareMagic; 64] = find_magics(Piece::Rook);

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
    let r1 = rng.gen::<u64>();
    let r2 = rng.gen::<u64>();
    let r3 = rng.gen::<u64>();
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

#[cfg(test)]
mod tests {
    use crate::{
        moves::gen::{
            magics::magic_index,
            sliders::{slider_attacks_from_square, BISHOP_MOVE_DIRECTIONS, ROOK_MOVE_DIRECTIONS},
        },
        position::{bitboard::Bitboard, piece::Piece, square::Square},
    };

    use super::{bitsets, find_magic};

    fn test_magics(piece: Piece) {
        let directions = match piece {
            Piece::Rook => &ROOK_MOVE_DIRECTIONS,
            Piece::Bishop => &BISHOP_MOVE_DIRECTIONS,
            _ => panic!("Invalid piece"),
        };

        for square in Square::list() {
            let magic = find_magic(*square, piece);

            let blockers_mask =
                slider_attacks_from_square(*square, directions, Bitboard::empty(), true);
            let bitsets = bitsets(blockers_mask);

            for bitset in bitsets {
                let index = magic_index(bitset, &magic);
                assert_eq!(
                    magic.attacks[index],
                    slider_attacks_from_square(*square, directions, bitset, false)
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
