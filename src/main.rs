use camel::moves::attacks::magics::init_magics;
use engine::uci_loop;

mod engine;

fn main() {
    init_magics();
    uci_loop();
}
