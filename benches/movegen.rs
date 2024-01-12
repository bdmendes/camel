use camel::{
    moves::gen::MoveStage,
    position::{
        fen::{FromFen, KIWIPETE_BLACK_FEN, KIWIPETE_WHITE_FEN},
        Position,
    },
};
use criterion::{criterion_group, criterion_main, Criterion};

fn generate_moves_kiwipete_white(c: &mut Criterion) {
    let kiwipete_position_white = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
    c.bench_function("generate_moves_kiwipete_white", |b| {
        b.iter(|| kiwipete_position_white.moves(MoveStage::All));
    });
}

fn generate_moves_kiwipete_black(c: &mut Criterion) {
    let kiwipete_position_black = Position::from_fen(KIWIPETE_BLACK_FEN).unwrap();
    c.bench_function("generate_moves_kiwipete_black", |b| {
        b.iter(|| kiwipete_position_black.moves(MoveStage::All));
    });
}

criterion_group!(movegen, generate_moves_kiwipete_white, generate_moves_kiwipete_black,);
criterion_main!(movegen);
