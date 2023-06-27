use std::{thread, time::Instant};

use camel::{
    evaluation::{Score, ValueScore},
    moves::{Move, MoveFlag},
    position::{square::Square, Position},
    search::{self, pvs::search, search_iter, table::SearchTable, Depth},
};

use crate::engine::Engine;

pub fn execute_position(new_position: &Position, engine: &mut Engine) {
    engine.position = new_position.clone();
}

pub fn execute_go(depth: u8, engine: &mut Engine) {
    let position = engine.position.clone();
    let mut table = SearchTable::new();

    thread::spawn(move || {
        search_iter(&position, depth as Depth, &mut table);
    });
}
