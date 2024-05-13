use camel::{
    moves::gen::MoveStage,
    position::{fen::FromFen, Position},
    search::see,
};
use criterion::{criterion_group, criterion_main, Criterion};

fn see_1(c: &mut Criterion) {
    let position =
        Position::from_fen("rnbqkb1r/1p1p1ppp/p3pn2/8/2PNP3/8/PP3PPP/RNBQKB1R w KQkq - 1 6")
            .unwrap();
    let moves = position.moves(MoveStage::All);
    let mov = moves.iter().find(|mov| mov.to_string() == "d4e6").unwrap();

    c.bench_function("see_1", |b| {
        b.iter(|| see::see::<false>(*mov, &position.board));
    });
}

fn see_2(c: &mut Criterion) {
    let position =
        Position::from_fen("rnbqkb1r/1p1p1ppp/p3p3/8/2PNn3/2N5/PP3PPP/R1BQKB1R w KQkq - 0 7")
            .unwrap();
    let moves = position.moves(MoveStage::All);
    let mov = moves.iter().find(|mov| mov.to_string() == "c3e4").unwrap();

    c.bench_function("see_2", |b| {
        b.iter(|| see::see::<false>(*mov, &position.board));
    });
}

criterion_group!(see, see_1, see_2,);
criterion_main!(see);
