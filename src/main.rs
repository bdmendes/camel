use engine::uci_loop;
use moves::attacks::magics::init_magics;

mod engine;
pub mod evaluation;
pub mod moves;
pub mod position;
pub mod search;

fn main() {
    init_magics();
    uci_loop();
}
