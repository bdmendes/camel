use moves::attacks::magics::{BISHOP_MAGICS, ROOK_MAGICS};
use once_cell::sync::Lazy;

mod moves;
mod position;

fn main() {
    Lazy::force(&ROOK_MAGICS);
    Lazy::force(&BISHOP_MAGICS);
}
