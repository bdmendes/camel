use std::time::Duration;

use camel::evaluation::position::endgame_ratio;

use crate::position::{Color, Position};

fn expected_remaining_moves(position: &Position) -> u32 {
    let endgame_ratio = endgame_ratio(position);
    let midgame_ratio = 255 - endgame_ratio;
    let expected = midgame_ratio as f32 / 5.0;
    std::cmp::max(10, expected as u32)
}

fn get_duration_based_on_moves(position: &Position, time: Duration) -> Duration {
    let number_available_moves = position.moves::<false>().len();

    if number_available_moves <= 1 {
        return Duration::from_millis(100);
    }

    let number_available_moves = number_available_moves + 1;
    let cof = 10 + (std::cmp::min(number_available_moves / 3, 10)) as u32;
    let expected_remaining = expected_remaining_moves(position);
    cof * time / (expected_remaining * 20)
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

    let standard_move_time = get_duration_based_on_moves(position, our_duration);

    if let Some(our_increment) = our_increment {
        if our_increment > Duration::from_millis(100) {
            let new_move_time = standard_move_time + our_increment - Duration::from_millis(100);
            if new_move_time < our_duration {
                return new_move_time - Duration::from_millis(100);
            }
        }
    }

    standard_move_time
}
