use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn start_position_move_generation(c: &mut Criterion) {
    c.bench_function("start position move generation", |b| {
        b.iter(|| {
            let position = black_box(camel::position::Position::new());
            black_box(camel::position::moves::legal_moves(
                &position,
                position.to_move,
            ));
        })
    });
}

criterion_group!(benches, start_position_move_generation);
criterion_main!(benches);
