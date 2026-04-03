use crate::position::Position;
use std::time::Duration;

fn get_duration_based_on_moves(position: &Position, time: Duration) -> Duration {
    let parabole_function = |x: f32| 0.01 * (150.0 - (x - 20.0) * (x - 20.0));
    let parabole_factor = parabole_function(position.fullmove_number() as f32);
    (time / 20).mul_f32(parabole_factor.clamp(0.5, 1.0))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn duration_around_move20() {
        let midgame = Position::from_str("3q2kr/p1r2pp1/2B1p2p/2Q1N2b/3PPn1P/5P2/PP6/2KR3R b - - 2 21").unwrap();
        let endgame = Position::from_str("2n5/6pk/8/3pp2p/7P/5P2/2KR4/8 b - - 1 46").unwrap();
        let time = Duration::from_secs(60);

        let mid_duration = get_duration_based_on_moves(&midgame, time);
        let end_duration = get_duration_based_on_moves(&endgame, time);

        assert!(mid_duration > end_duration, "expected more time for midgame than endgame");
        assert!(
            mid_duration < time / 10,
            "expected midgame duration to be less than one-tenth of total time"
        );
        assert!(
            end_duration < time / 10,
            "expected endgame duration to be less than one-tenth of total time"
        );
    }
}
