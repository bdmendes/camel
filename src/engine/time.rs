use camel::{
    moves::gen::MoveStage,
    position::{Color, Position},
};
use std::time::Duration;

const TYPICAL_GAME_MOVES: u16 = 50;

fn get_duration_based_on_moves(position: &Position, time: Duration) -> Duration {
    let expected_remaining_moves =
        std::cmp::max(10, TYPICAL_GAME_MOVES.saturating_sub(position.fullmove_number));
    let regular_time = time / expected_remaining_moves as u32;

    let parabole_function = |x: f32| 0.01 * (200.0 - (x - 20.0) * (x - 20.0));
    let parabole_factor = parabole_function(position.fullmove_number as f32);

    regular_time.mul_f32(parabole_factor.max(0.8))
}

pub fn get_duration(
    position: &Position,
    white_time: Duration,
    black_time: Duration,
    white_increment: Option<Duration>,
    black_increment: Option<Duration>,
) -> Duration {
    let our_duration = match position.side_to_move {
        Color::White => white_time,
        Color::Black => black_time,
    };
    let our_increment = match position.side_to_move {
        Color::White => white_increment,
        Color::Black => black_increment,
    };

    let moves_cof = position.moves(MoveStage::All).len() as f32 / 40.0;
    let mut standard_move_time = get_duration_based_on_moves(position, our_duration)
        .mul_f32(moves_cof)
        .max(Duration::from_millis(10));

    if standard_move_time < Duration::from_secs(1) {
        standard_move_time /= 2;
    }

    if let Some(our_increment) = our_increment {
        let new_move_time = standard_move_time + our_increment;
        if new_move_time < our_duration {
            return new_move_time;
        }
    }

    standard_move_time
}
