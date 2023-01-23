use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn double_attack_position_search(c: &mut Criterion) {
    let double_attack_fen = "2kr3r/ppp2q2/4p2p/3nn3/2P3p1/1B5Q/P1P2PPP/R1B1K2R w KQ - 0 17";
    c.bench_function("double attack position search", |b| {
        b.iter(|| {
            let position =
                black_box(camel::position::Position::from_fen(double_attack_fen).unwrap());
            black_box(camel::search::Searcher::new().search(&position, 2));
        })
    });
}

criterion_group!(benches, double_attack_position_search);
criterion_main!(benches);
