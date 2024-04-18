use camel::position::{Color, Position};
use std::{ops::Div, time::Duration};

const TYPICAL_GAME_MOVES: u16 = 40;
const AVERAGE_NUMBER_OF_PIECES: u16 = 24;

fn get_duration_based_on_moves(position: &Position, time: Duration) -> Duration {
    // Assume the game will have a typical duration and divide the time by the expected remaining moves.
    let expected_remaining_moves =
        std::cmp::max(10, TYPICAL_GAME_MOVES.saturating_sub(position.fullmove_number));
    let regular_time = time / expected_remaining_moves as u32;

    // Positions tend to get more tense around move 20, so we want to increase the time around that move.
    let parabole_function = |x: f32| 0.01 * (200.0 - (x - 20.0) * (x - 20.0));
    let move_number_factor = parabole_function(position.fullmove_number as f32).max(0.9).min(1.5);

    // Positions with less space tend to be more complex, so increase the time for those.
    let number_of_pieces = position.board.occupancy_bb_all().count_ones() as f32;
    let number_of_pieces_factor =
        number_of_pieces.div(AVERAGE_NUMBER_OF_PIECES as f32).max(0.9).min(1.2);

    regular_time.mul_f32(move_number_factor).mul_f32(number_of_pieces_factor)
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
        standard_move_time += standard_move_time / 4;
    }

    if standard_move_time < Duration::from_secs(1) {
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
