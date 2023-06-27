use camel::position::Position;

use crate::engine::Engine;

pub fn execute_position(new_position: &Position, engine: &mut Engine) {
    engine.position = new_position.clone();
}

pub fn execute_go(depth: u8, engine: &mut Engine) {
    todo!()
}
