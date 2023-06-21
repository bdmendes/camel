use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    thread,
};

use tinyrand::{Rand, Seeded, StdRand};
use tinyrand_std::ClockSeed;

use crate::position::{bitboard::Bitboard, board::Piece, square::Square};

use super::sliders::{slider_attacks_from_square, BISHOP_MOVE_DIRECTIONS, ROOK_MOVE_DIRECTIONS};

pub static ROOK_MAGICS: OnceLock<[SquareMagic; 64]> = OnceLock::new();
pub static BISHOP_MAGICS: OnceLock<[SquareMagic; 64]> = OnceLock::new();

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

fn sparse_random() -> Bitboard {
    let seed = ClockSeed::default().next_u64();
    let mut rand = StdRand::seed(seed);

    let r1 = rand.next_u64();
    let r2 = rand.next_u64();
    let r3 = rand.next_u64();

    Bitboard::new(r1 & r2 & r3)
}

fn find_magic(square: Square, piece: Piece) -> SquareMagic {
    let blockers_mask = slider_attacks_from_square::<true>(
        square,
        match piece {
            Piece::Rook => &ROOK_MOVE_DIRECTIONS,
            Piece::Bishop => &BISHOP_MOVE_DIRECTIONS,
            _ => unreachable!(),
        },
        None,
    );

    let shift = blockers_mask.count_ones() as u8;
    let bitsets = bitsets(blockers_mask);

    let mut cached_moves: HashMap<Bitboard, Bitboard> = HashMap::new();

    loop {
        let mut magic = SquareMagic {
            blockers_mask,
            shift,
            magic_number: sparse_random(),
            attacks: vec![Bitboard::new(0); 1 << shift],
        };

        let mut found_collision = false;
        let mut used = vec![false; 1 << shift];

        for bitset in bitsets.iter() {
            let index = magic_index(&magic, *bitset);
            let moves = *cached_moves.entry(*bitset).or_insert_with(|| {
                slider_attacks_from_square::<false>(
                    square,
                    if piece == Piece::Rook {
                        &ROOK_MOVE_DIRECTIONS
                    } else {
                        &BISHOP_MOVE_DIRECTIONS
                    },
                    Some(*bitset),
                )
            });

            if used[index] && magic.attacks[index] != moves {
                found_collision = true;
                break;
            }

            used[index] = true;
            magic.attacks[index] = moves;
        }

        if !found_collision {
            return magic;
        }
    }
}

fn find_magics(piece: Piece) -> [SquareMagic; 64] {
    let magics = Arc::new(Mutex::new(array_init::array_init(|_| SquareMagic::default())));

    let mut handles = vec![];

    for square in 0..64 {
        let magics_ref = Arc::clone(&magics);
        let handle = thread::spawn(move || {
            let magic = find_magic(Square::try_from(square).unwrap(), piece);
            magics_ref.lock().unwrap()[square as usize] = magic;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(magics).unwrap().into_inner().unwrap()
}

pub fn magic_index(magic: &SquareMagic, occupancy: Bitboard) -> usize {
    let blockers = occupancy & magic.blockers_mask;
    let hash = blockers.wrapping_mul(magic.magic_number.raw());
    (hash >> (64 - magic.shift)) as usize
}

pub fn init_magics() {
    ROOK_MAGICS.set(find_magics(Piece::Rook)).unwrap();
    BISHOP_MAGICS.set(find_magics(Piece::Bishop)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn magic_finders_do_not_hang() {
        find_magics(Piece::Rook);
        find_magics(Piece::Bishop);
    }
}
