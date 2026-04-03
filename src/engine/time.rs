use crate::position::Position;
use std::time::Duration;

const TYPICAL_GAME_MOVES: u16 = 50;

fn get_duration_based_on_moves(position: &Position, time: Duration) -> Duration {
    let expected_remaining_moves = TYPICAL_GAME_MOVES.saturating_sub(position.fullmove_number()).max(10);
    let regular_time = time / expected_remaining_moves as u32;

    let parabole_function = |x: f32| 0.01 * (150.0 - (x - 20.0) * (x - 20.0));
    let parabole_factor = parabole_function(position.fullmove_number() as f32);

    regular_time.mul_f32(parabole_factor.max(0.8))
}

pub fn get_duration(position: &Position, time: Duration, increment: Option<Duration>, ponder: bool) -> Duration {
    let mut standard_move_time = get_duration_based_on_moves(position, time);

    if ponder {
        standard_move_time += standard_move_time / 4;
    }

    if standard_move_time < Duration::from_secs(1) {
        standard_move_time /= 2;
    }

    if let Some(increment) = increment {
        let new_move_time = standard_move_time + increment.mul_f32(0.9);
        if new_move_time < time {
            return new_move_time;
        }
    }

    standard_move_time
}
