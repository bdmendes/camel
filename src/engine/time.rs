use camel::{
    evaluation::{Evaluable, ValueScore},
    position::{board::Piece, Color, Position},
    search::{constraint::SearchConstraint, pvs::quiesce},
};
use std::time::Duration;

const TYPICAL_GAME_MOVES: u16 = 50;

fn get_duration_based_on_moves(position: &Position, time: Duration) -> Duration {
    let expected_remaining_moves =
        std::cmp::max(15, TYPICAL_GAME_MOVES.saturating_sub(position.fullmove_number));
    let regular_time = time / expected_remaining_moves as u32;

    let parabole_function = |x: f32| 0.01 * (150.0 - (x - 20.0) * (x - 20.0));
    let parabole_factor = parabole_function(position.fullmove_number as f32);

    regular_time.mul_f32(parabole_factor.max(0.8))
}

pub fn get_duration(
    position: &Position,
    white_time: Duration,
    black_time: Duration,
    white_increment: Option<Duration>,
    black_increment: Option<Duration>,
    ponder: bool,
) -> Duration {
    let our_duration = match position.side_to_move {
        Color::White => white_time,
        Color::Black => black_time,
    };
    let our_increment = match position.side_to_move {
        Color::White => white_increment,
        Color::Black => black_increment,
    };

    let mut standard_move_time = get_duration_based_on_moves(position, our_duration);

    if ponder {
        // Assume we'll have some ponderhits, and can be more aggressive
        // in time management.
        standard_move_time += standard_move_time / 4;
    }

    if quiesce(position, ValueScore::MIN + 1, ValueScore::MAX, &SearchConstraint::default(), 0)
        .0
        .abs()
        > Piece::Knight.value()
    {
        // This position is probably not very interesting. Let's speed up.
        standard_move_time /= 2;
    }

    if let Some(our_increment) = our_increment {
        let new_move_time = standard_move_time + our_increment.mul_f32(0.9);
        if new_move_time < our_duration {
            return new_move_time;
        }
    }

    standard_move_time
}
