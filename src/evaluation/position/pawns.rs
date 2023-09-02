use crate::{
    evaluation::ValueScore,
    position::{Color, Position},
};

pub fn evaluate_pawn_structure(position: &Position) -> ValueScore {
    let mut score = 0;
    let white_pawn_structure = position.board.pawn_structure(Color::White);
    let black_pawn_structure = position.board.pawn_structure(Color::Black);

    for file in 0..8 {
        for color in &[Color::White, Color::Black] {
            let structure =
                if *color == Color::White { &white_pawn_structure } else { &black_pawn_structure };

            let is_isolated = structure[file] > 0
                && (file == 0 || structure[file - 1] == 0)
                && (file == 7 || structure[file + 1] == 0);
            if is_isolated {
                score -= 10 * color.sign();
            }

            let doubled_penalty = match structure[file] {
                0 => 0,
                1 => 0,
                2 => 10,
                _ => 30,
            };
            score -= doubled_penalty * color.sign() * (if is_isolated { 3 } else { 1 });
        }
    }

    score
}
