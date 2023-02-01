use std::time::Duration;

use crate::{
    evaluation::evaluate_position,
    position::{Color, Position},
};

fn expected_remaining_moves(position: &Position) -> u32 {
    const TYPICAL_MOVES_PER_GAME: u16 = 50;
    if position.info.full_move_number > TYPICAL_MOVES_PER_GAME - 10 {
        return 10;
    }
    (TYPICAL_MOVES_PER_GAME - position.info.full_move_number) as u32
}

fn get_duration_based_on_eval(position: &Position, time: Duration) -> Duration {
    let our_eval = evaluate_position(position, false, true);

    let cof = if our_eval > 300 {
        20 - (std::cmp::min(our_eval / 100, 10)) as u32
    } else if our_eval < -300 {
        20
    } else {
        std::cmp::min(15 + (position.info.full_move_number / 5) as u32, 20)
    };

    let expected_remaining = expected_remaining_moves(position);
    cof * time / (expected_remaining * 20)
}

pub fn get_duration(
    position: &Position,
    white_time: Duration,
    black_time: Duration,
) -> Duration {
    let our_duration = match position.info.to_move {
        Color::White => white_time,
        Color::Black => black_time,
    };

    // Play fast if we have no time
    if our_duration < Duration::from_secs(1) {
        return Duration::from_millis(10);
    }

    // Play fast on the first move
    if position.info.full_move_number == 1 {
        return Duration::from_millis(500);
    }

    get_duration_based_on_eval(position, our_duration)
}
