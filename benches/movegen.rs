use camel::{
    moves::gen::perft,
    position::{
        fen::{KIWIPETE_BLACK_FEN, KIWIPETE_WHITE_FEN, START_FEN},
        Position,
    },
};
use criterion::{criterion_group, criterion_main, Criterion};

fn generate_moves_kiwipete_white(c: &mut Criterion) {
    let kiwipete_position_white = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
    c.bench_function("generate_moves_kiwipete_white", |b| {
        b.iter(|| kiwipete_position_white.moves::<false>());
    });
}

fn generate_moves_kiwipete_black(c: &mut Criterion) {
    let kiwipete_position_black = Position::from_fen(KIWIPETE_BLACK_FEN).unwrap();
    c.bench_function("generate_moves_kiwipete_black", |b| {
        b.iter(|| kiwipete_position_black.moves::<false>());
    });
}

fn perft_start_position(c: &mut Criterion) {
    let start_position = Position::from_fen(START_FEN).unwrap();
    c.bench_function("perft_start_position", |b| {
        b.iter(|| perft::<false, false, true>(&start_position, 3));
    });
}

criterion_group!(
    movegen,
    generate_moves_kiwipete_white,
    generate_moves_kiwipete_black,
    perft_start_position
);
criterion_main!(movegen);
