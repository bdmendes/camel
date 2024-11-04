use camel::position::{color::Color, piece::Piece, square::Square, Position};

fn main() {
    divan::main();
}

#[divan::bench]
fn piece_color_at() {
    for _ in 0..=divan::black_box(1_000_000) {
        let mut position = divan::black_box(Position::default());
        position.set_square(Square::E4, Piece::Pawn, Color::White);
        let _ = divan::black_box(position.piece_color_at(Square::E4));
    }
}
