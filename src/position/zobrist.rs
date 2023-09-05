use once_cell::sync::Lazy;
use rand::Rng;

pub type ZobristHash = u64;

const ZOBRIST_NUMBERS_SIZE: usize = 12 * 64 + 2 + 65 + 16;

pub static ZOBRIST_NUMBERS: Lazy<[ZobristHash; ZOBRIST_NUMBERS_SIZE]> = Lazy::new(|| {
    let mut rng = rand::thread_rng();
    let mut numbers = [0; ZOBRIST_NUMBERS_SIZE];
    numbers.iter_mut().take(ZOBRIST_NUMBERS_SIZE).for_each(|n| *n = rng.gen());
    numbers
});
