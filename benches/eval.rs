use camel::{
    evaluation::position::evaluate_position,
    position::{
        fen::{KIWIPETE_WHITE_FEN, START_FEN},
        Position,
    },
};
use criterion::{criterion_group, criterion_main, Criterion};

fn eval_start(c: &mut Criterion) {
    let start_position = Position::from_fen(START_FEN).unwrap();
    c.bench_function("eval_start_position", |b| {
        b.iter(|| evaluate_position(&start_position));
    });
}

fn eval_kiwipete(c: &mut Criterion) {
    let kiwipete_position_white = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
    c.bench_function("eval_kiwipete", |b| {
        b.iter(|| evaluate_position(&kiwipete_position_white));
    });
}

criterion_group!(eval, eval_start, eval_kiwipete);
criterion_main!(eval);