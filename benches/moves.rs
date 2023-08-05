use camel::position::{
    fen::{KIWIPETE_BLACK_FEN, KIWIPETE_WHITE_FEN, START_FEN},
    Position,
};
use criterion::{criterion_group, criterion_main, Criterion};

fn gen_startpos(c: &mut Criterion) {
    let position = Position::from_fen(START_FEN).unwrap();
    c.bench_function("gen_startpos", |b| b.iter(|| position.moves::<false>()));
}

fn gen_kiwipete_white(c: &mut Criterion) {
    let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
    c.bench_function("gen_kiwipete_white", |b| b.iter(|| position.moves::<false>()));
}

fn gen_kiwipete_black(c: &mut Criterion) {
    let position = Position::from_fen(KIWIPETE_BLACK_FEN).unwrap();
    c.bench_function("gen_kiwipete_black", |b| b.iter(|| position.moves::<false>()));
}

fn make_startpos(c: &mut Criterion) {
    let position = Position::from_fen(START_FEN).unwrap();
    let mov = position.moves::<false>().into_iter().find(|m| m.to_string() == "e2e4").unwrap();
    c.bench_function("make_startpos", |b| b.iter(|| position.make_move(mov)));
}

fn make_kiwipete_white(c: &mut Criterion) {
    let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
    let mov = position.moves::<false>().into_iter().find(|m| m.to_string() == "e1g1").unwrap();
    c.bench_function("make_kiwipete_white", |b| b.iter(|| position.make_move(mov)));
}

fn make_kiwipete_black(c: &mut Criterion) {
    let position = Position::from_fen(KIWIPETE_BLACK_FEN).unwrap();
    let mov = position.moves::<false>().into_iter().find(|m| m.to_string() == "a6e2").unwrap();
    c.bench_function("make_kiwipete_black", |b| b.iter(|| position.make_move(mov)));
}

criterion_group!(
    moves_benches,
    gen_startpos,
    gen_kiwipete_white,
    gen_kiwipete_black,
    make_startpos,
    make_kiwipete_white,
    make_kiwipete_black
);
criterion_main!(moves_benches);
