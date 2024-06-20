use bitmap::Bitmap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_add(c: &mut Criterion) {
    let left = Bitmap::from_iter(&[0, 1, 2, 6, 9, 10, 25, 90, 91, 150, 2000]);
    let right = Bitmap::from_iter(&[0, 1, 3, 4, 9, 10, 29, 90, 91, 150, 3000]);
    c.bench_function("small", |b| b.iter(|| black_box(left.clone() & &right)));

    let left = Bitmap::from_iter((0..200).chain(1000..2000).step_by(3).step_by(5));
    let right = Bitmap::from_iter((100..300).chain(1000..2000).step_by(5).step_by(3));
    c.bench_function("medium", |b| b.iter(|| black_box(left.clone() & &right)));

    let left = Bitmap::from_iter(
        (0..20_000)
            .step_by(2)
            .step_by(3)
            .chain(50_000..60_000)
            .step_by(3)
            .step_by(5),
    );
    let right = Bitmap::from_iter(
        (0..20_000)
            .step_by(3)
            .step_by(2)
            .chain(50_000..60_000)
            .step_by(5)
            .step_by(3),
    );
    c.bench_function("large", |b| b.iter(|| black_box(left.clone() & &right)));
}

criterion_group!(benches, bench_add);
criterion_main!(benches);
