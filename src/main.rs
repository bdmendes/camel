use camel::*;
use engine::uci_loop;
use moves::attacks::magics::init_magics;

mod engine;

fn main() {
    init_magics();
    uci_loop();
}
