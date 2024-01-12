use camel::{
    evaluation::Evaluable,
    position::{
        fen::{FromFen, KIWIPETE_WHITE_FEN, START_FEN},
        Position,
    },
};
use criterion::{criterion_group, criterion_main, Criterion};

fn eval_start(c: &mut Criterion) {
    let start_position = Position::from_fen(START_FEN).unwrap();
    c.bench_function("eval_start_position", |b| {
        b.iter(|| start_position.value());
    });
}

fn eval_kiwipete(c: &mut Criterion) {
    let kiwipete_position_white = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
    c.bench_function("eval_kiwipete", |b| {
        b.iter(|| kiwipete_position_white.value());
    });
}

criterion_group!(eval, eval_start, eval_kiwipete);
criterion_main!(eval);
